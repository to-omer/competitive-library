use crate::error::{ParseError, ParseResult};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens as _};
use std::path::{Path, PathBuf};
use syn::{
    visit_mut::{self, VisitMut},
    Attribute, File, ItemMod,
};

pub fn parse_file_recursive(path: PathBuf) -> ParseResult<File> {
    let mut ext = ExtractAst { path, error: None };
    let mut ast = parse_file_from_path(&ext.path)?;
    ext.visit_file_mut(&mut ast);
    match ext.error {
        Some(err) => Err(err),
        _ => Ok(ast),
    }
}

#[derive(Debug)]
struct ExtractAst {
    path: PathBuf,
    error: Option<ParseError>,
}

impl ExtractAst {
    fn find_mod_file(&self, node: &ItemMod) -> ParseResult<PathBuf> {
        let mod_name = node.ident.to_string();
        let mod_path = self.path.with_file_name(&mod_name);
        if let Some(pathstr) = find_pathstr_from_attrs(&node.attrs) {
            let mod_path = self.path.with_file_name(pathstr);
            if mod_path.exists() {
                return Ok(mod_path);
            }
        }
        let path1 = mod_path.with_extension("rs");
        let path2 = mod_path.join("mod.rs");
        if path1.exists() {
            Ok(path1)
        } else if path2.exists() {
            Ok(path2)
        } else {
            Err(ParseError::ModuleNotFound(mod_name))
        }
    }

    fn expand_file(&self, node: &mut ItemMod) -> ParseResult<PathBuf> {
        let path = self.find_mod_file(&node)?;
        let ast = parse_file_from_path(&path)?;

        node.attrs.extend(ast.attrs);
        let mut tokens = TokenStream::new();
        for attr in node.attrs.iter() {
            if attr.style == syn::AttrStyle::Outer {
                attr.to_tokens(&mut tokens);
            }
        }
        node.vis.to_tokens(&mut tokens);
        node.mod_token.to_tokens(&mut tokens);
        node.ident.to_tokens(&mut tokens);

        let mut file_items = TokenStream::new();
        for attr in node.attrs.iter() {
            if attr.style != syn::AttrStyle::Outer {
                attr.to_tokens(&mut file_items);
            }
        }
        for item in ast.items.iter() {
            item.to_tokens(&mut file_items);
        }
        let braced = quote! { { #file_items } };
        braced.to_tokens(&mut tokens);

        let item_mod = syn::parse2::<ItemMod>(tokens)?;
        *node = item_mod;
        Ok(path)
    }
}

impl VisitMut for ExtractAst {
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
}

fn parse_file_from_path<P: AsRef<Path>>(path: P) -> ParseResult<File> {
    use std::io::Read as _;
    let mut content = String::new();
    let mut file = std::fs::File::open(&path)?;
    file.read_to_string(&mut content)?;
    Ok(syn::parse_file(&content)?)
}

fn find_pathstr_from_attrs(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .filter(|attr| attr.style == syn::AttrStyle::Outer)
        .filter_map(|attr| attr.parse_meta().ok())
        .filter(|meta| {
            meta.path()
                .get_ident()
                .map(|id| *id == "path")
                .unwrap_or_default()
        })
        .filter_map(|meta| {
            if let syn::Meta::NameValue(metanv) = meta {
                if let syn::Lit::Str(litstr) = metanv.lit {
                    return Some(litstr.value());
                }
            }
            None
        })
        .next()
}
