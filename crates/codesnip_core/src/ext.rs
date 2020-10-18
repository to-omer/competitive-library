use syn::{parse::Parser, parse_str, Attribute, Item, Path};

pub trait AttributeExt {
    fn parse_args_empty_with<F: Parser>(&self, parser: F) -> syn::Result<F::Output>;
}

pub trait ItemExt {
    fn get_attributes(&self) -> Option<&[Attribute]>;
    fn get_attributes_mut(&mut self) -> Option<&mut Vec<Attribute>>;
    fn get_default_name(&self) -> Option<String>;
    fn is_mod(&self) -> bool;
}

pub trait PathExt {
    fn is_codesnip_entry(&self) -> bool;
    fn is_codesnip_skip(&self) -> bool;
}

impl AttributeExt for Attribute {
    fn parse_args_empty_with<F: Parser>(&self, parser: F) -> syn::Result<F::Output> {
        if self.tokens.is_empty() {
            parser.parse2(self.tokens.clone())
        } else {
            self.parse_args_with(parser)
        }
    }
}

impl ItemExt for Item {
    fn get_attributes(&self) -> Option<&[Attribute]> {
        Some(match self {
            Item::Const(it) => &it.attrs,
            Item::Enum(it) => &it.attrs,
            Item::ExternCrate(it) => &it.attrs,
            Item::Fn(it) => &it.attrs,
            Item::ForeignMod(it) => &it.attrs,
            Item::Impl(it) => &it.attrs,
            Item::Macro(it) => &it.attrs,
            Item::Macro2(it) => &it.attrs,
            Item::Mod(it) => &it.attrs,
            Item::Static(it) => &it.attrs,
            Item::Struct(it) => &it.attrs,
            Item::Trait(it) => &it.attrs,
            Item::TraitAlias(it) => &it.attrs,
            Item::Type(it) => &it.attrs,
            Item::Union(it) => &it.attrs,
            Item::Use(it) => &it.attrs,
            _ => return None,
        })
    }

    fn get_attributes_mut(&mut self) -> Option<&mut Vec<Attribute>> {
        Some(match self {
            Item::Const(it) => &mut it.attrs,
            Item::Enum(it) => &mut it.attrs,
            Item::ExternCrate(it) => &mut it.attrs,
            Item::Fn(it) => &mut it.attrs,
            Item::ForeignMod(it) => &mut it.attrs,
            Item::Impl(it) => &mut it.attrs,
            Item::Macro(it) => &mut it.attrs,
            Item::Macro2(it) => &mut it.attrs,
            Item::Mod(it) => &mut it.attrs,
            Item::Static(it) => &mut it.attrs,
            Item::Struct(it) => &mut it.attrs,
            Item::Trait(it) => &mut it.attrs,
            Item::TraitAlias(it) => &mut it.attrs,
            Item::Type(it) => &mut it.attrs,
            Item::Union(it) => &mut it.attrs,
            Item::Use(it) => &mut it.attrs,
            _ => return None,
        })
    }

    fn get_default_name(&self) -> Option<String> {
        Some(
            match self {
                Item::Const(it) => &it.ident,
                Item::Enum(it) => &it.ident,
                Item::ExternCrate(it) => &it.ident,
                Item::Fn(it) => &it.sig.ident,
                // Item::ForeignMod(it) => &it,
                // Item::Impl(it) => &it,
                Item::Macro(it) => return it.ident.as_ref().map(|id| id.to_string()),
                Item::Macro2(it) => &it.ident,
                Item::Mod(it) => &it.ident,
                Item::Static(it) => &it.ident,
                Item::Struct(it) => &it.ident,
                Item::Trait(it) => &it.ident,
                Item::TraitAlias(it) => &it.ident,
                Item::Type(it) => &it.ident,
                Item::Union(it) => &it.ident,
                // Item::Use(it) => &it,
                _ => return None,
            }
            .to_string(),
        )
    }

    fn is_mod(&self) -> bool {
        matches!(self, Item::Mod(_))
    }
}

thread_local! {
    static SNIPPET_ENTRY: Path = parse_str::<Path>("codesnip::entry").unwrap();
    static SNIPPET_SKIP: Path = parse_str::<Path>("codesnip::skip").unwrap();
}

impl PathExt for Path {
    fn is_codesnip_entry(&self) -> bool {
        SNIPPET_ENTRY.with(|path| self == path)
    }

    fn is_codesnip_skip(&self) -> bool {
        SNIPPET_SKIP.with(|path| self == path)
    }
}

#[test]
fn test_is_codesnip() {
    assert!(&parse_str::<Path>("codesnip::entry")
        .unwrap()
        .is_codesnip_entry());
    assert!(!&parse_str::<Path>("entry").unwrap().is_codesnip_entry());
    assert!(!&parse_str::<Path>("::codesnip::entry")
        .unwrap()
        .is_codesnip_entry());
    assert!(!&parse_str::<Path>("codesnip").unwrap().is_codesnip_entry());
    assert!(!&parse_str::<Path>("::entry").unwrap().is_codesnip_entry());
    assert!(!&parse_str::<Path>("rustfmt::entry")
        .unwrap()
        .is_codesnip_entry());
}
