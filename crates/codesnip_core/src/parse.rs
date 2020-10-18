use crate::ItemExt as _;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens as _};
use std::path::{Path, PathBuf};
use syn::{
    parse2, parse_file,
    visit_mut::{self, VisitMut},
    AttrStyle, Attribute, File, Item, ItemMod, Lit, Meta, NestedMeta,
};
use Error::{FileNotFound, ModuleNotFound, ParseFile};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to parse ")]
    ParseFile(PathBuf, #[source] syn::Error),
    #[error("Module `{0}` not found where `{}`.", .1.display())]
    ModuleNotFound(String, PathBuf),
    #[error("File `{}` not found.", .0.display())]
    FileNotFound(PathBuf, #[source] std::io::Error),
}

pub fn parse_file_recursive(path: PathBuf, cfg: &[Meta]) -> Result<File, Error> {
    let mut ext = ExtractAst {
        path,
        error: None,
        cfg,
    };
    let mut ast = parse_file_from_path(&ext.path)?;
    ext.visit_file_mut(&mut ast);
    match ext.error {
        Some(err) => Err(err),
        _ => Ok(ast),
    }
}

#[derive(Debug)]
struct ExtractAst<'c> {
    path: PathBuf,
    error: Option<Error>,
    cfg: &'c [Meta],
}

impl ExtractAst<'_> {
    fn find_mod_file(&self, node: &ItemMod) -> Result<PathBuf, Error> {
        let mod_name = node.ident.to_string();
        let mod_path = self.path.with_file_name(&mod_name);
        if let Some(pathstr) = find_pathstr_from_attrs(&node.attrs) {
            let mod_path = self.path.with_file_name(pathstr);
            if mod_path.exists() {
                Ok(mod_path)
            } else {
                Err(ModuleNotFound(mod_name, self.path.to_path_buf()))
            }
        } else {
            let path1 = mod_path.with_extension("rs");
            let path2 = mod_path.join("mod.rs");
            if path1.exists() {
                Ok(path1)
            } else if path2.exists() {
                Ok(path2)
            } else {
                Err(ModuleNotFound(mod_name, self.path.to_path_buf()))
            }
        }
    }

    fn expand_file(&self, node: &mut ItemMod) -> Result<PathBuf, Error> {
        let path = self.find_mod_file(&node)?;
        let ast = parse_file_from_path(&path)?;

        node.attrs.extend(ast.attrs);
        let mut tokens = TokenStream::new();
        for attr in node.attrs.iter() {
            if attr.style == AttrStyle::Outer {
                attr.to_tokens(&mut tokens);
            }
        }
        node.vis.to_tokens(&mut tokens);
        node.mod_token.to_tokens(&mut tokens);
        node.ident.to_tokens(&mut tokens);

        let mut file_items = TokenStream::new();
        for attr in node.attrs.iter() {
            if attr.style != AttrStyle::Outer {
                attr.to_tokens(&mut file_items);
            }
        }
        for item in ast.items.iter() {
            item.to_tokens(&mut file_items);
        }
        let braced = quote! { { #file_items } };
        braced.to_tokens(&mut tokens);

        let item_mod = parse2::<ItemMod>(tokens).expect("failed to parse no-inline `mod`");
        *node = item_mod;
        Ok(path)
    }
}

impl VisitMut for ExtractAst<'_> {
    fn visit_item_mod_mut(&mut self, node: &mut ItemMod) {
        let cur = self.path.clone();
        if node.content.is_none() {
            match self.expand_file(node) {
                Ok(path) => {
                    self.path = path;
                }
                Err(err) => {
                    self.error.get_or_insert(err);
                }
            }
        } else {
            let pathstr = if let Some(pathstr) = find_pathstr_from_attrs(&node.attrs) {
                pathstr
            } else {
                node.ident.to_string()
            };
            let path = Path::new(&pathstr);
            self.path = self.path.with_file_name(path.join("mod.rs"));
        }
        visit_mut::visit_item_mod_mut(self, node);
        self.path = cur;
    }
    fn visit_item_mut(&mut self, node: &mut Item) {
        let mut is_skip = false;
        if let Some(attrs) = node.get_attributes_mut() {
            if !check_cfg(attrs, &self.cfg) {
                is_skip = true;
            } else {
                flatten_cfg_attr(attrs, &self.cfg);
            }
        }
        if is_skip {
            *node = Item::Verbatim(TokenStream::new());
        } else {
            visit_mut::visit_item_mut(self, node);
        }
    }
}

fn parse_file_from_path<P: AsRef<Path>>(path: P) -> Result<File, Error> {
    use std::io::Read as _;
    let mut content = String::new();
    let mut file =
        std::fs::File::open(&path).map_err(|err| FileNotFound(path.as_ref().to_path_buf(), err))?;
    file.read_to_string(&mut content)?;
    Ok(parse_file(&content).map_err(|err| ParseFile(path.as_ref().to_path_buf(), err))?)
}

fn find_pathstr_from_attrs(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer)
        .filter_map(|attr| attr.parse_meta().ok())
        .filter(|meta| {
            meta.path()
                .get_ident()
                .map(|id| *id == "path")
                .unwrap_or_default()
        })
        .filter_map(|meta| {
            if let Meta::NameValue(metanv) = meta {
                if let Lit::Str(litstr) = metanv.lit {
                    return Some(litstr.value());
                }
            }
            None
        })
        .next()
}

fn check_cfg(attrs: &mut Vec<Attribute>, cfg: &[Meta]) -> bool {
    let mut next = Vec::new();
    let mut cond = true;
    for attr in attrs.drain(..) {
        if let Ok(meta) = attr.parse_meta() {
            if meta.path().is_ident("cfg") {
                if let Meta::List(list) = meta {
                    if let Some(NestedMeta::Meta(pred)) = list.nested.first() {
                        cond &= cfg_condition(pred, cfg);
                        continue;
                    }
                }
            }
        }
        next.push(attr);
    }
    *attrs = next;
    cond
}

fn flatten_cfg_attr(attrs: &mut Vec<Attribute>, cfg: &[Meta]) {
    let mut next = Vec::new();
    for attr in attrs.drain(..) {
        if let Ok(meta) = attr.parse_meta() {
            if meta.path().is_ident("cfg_attr") {
                if let Meta::List(list) = meta {
                    let mut it = list.nested.iter();
                    if let Some(NestedMeta::Meta(pred)) = it.next() {
                        if cfg_condition(pred, cfg) {
                            next.extend(it.map(to_attribute));
                            continue;
                        }
                    }
                }
            }
        }
        next.push(attr);
    }
    *attrs = next;
}

fn to_attribute(meta: &NestedMeta) -> Attribute {
    let meta = meta.to_token_stream();
    let attr: Attribute = syn::parse_quote!(#[ #meta ]);
    attr
}

#[test]
fn test_to_attribute() {
    let attr: Attribute = syn::parse_quote!(#[codesnip::entry("name", inline)]);
    let attr = to_attribute(&NestedMeta::Meta(attr.parse_meta().unwrap()))
        .to_token_stream()
        .to_string();
    assert_eq!(
        attr.as_str(),
        r##"# [codesnip :: entry ("name" , inline)]"##
    );
}

fn cfg_condition(pred: &Meta, cfg: &[Meta]) -> bool {
    if let Some(id) = pred.path().get_ident() {
        match id.to_string().as_str() {
            "all" => {
                if let Meta::List(list) = pred {
                    return list.nested.iter().all(|nm| {
                        if let NestedMeta::Meta(pred) = nm {
                            cfg_condition(pred, cfg)
                        } else {
                            true
                        }
                    });
                }
            }
            "any" => {
                if let Meta::List(list) = pred {
                    return list.nested.iter().any(|nm| {
                        if let NestedMeta::Meta(pred) = nm {
                            cfg_condition(pred, cfg)
                        } else {
                            false
                        }
                    });
                }
            }
            "not" => {
                if let Meta::List(list) = pred {
                    if let Some(NestedMeta::Meta(pred)) = list.nested.first() {
                        return !cfg_condition(pred, cfg);
                    }
                }
            }
            _ => return cfg.iter().any(|spec| spec == pred),
        }
    }
    // If there is a parsing error, it may be skipped and succeeded.
    true
}
