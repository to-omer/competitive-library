use syn::{Attribute, Item};

pub fn get_attributes_of_item(node: &Item) -> Option<&[Attribute]> {
    Some(match node {
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

pub fn get_attributes_of_item_mut(node: &mut Item) -> Option<&mut Vec<Attribute>> {
    Some(match node {
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

pub fn get_default_name_of_item(node: &Item) -> Option<String> {
    Some(
        match node {
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
