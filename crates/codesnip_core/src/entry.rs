use crate::ext::ItemExt as _;
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Comma, Paren},
    Error, Ident, Item, LitStr,
};

#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub name: String,
    pub include: Vec<String>,
    pub inline: bool,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct EntryArgs {
    pub args: Punctuated<EntryArg, Comma>,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum EntryArg {
    Name(EntryArgName),
    Include(EntryArgInclude),
    Inline(EntryArgInline),
    NoInline(EntryArgNoInline),
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct EntryArgName {
    pub name_token: Option<(Ident, token::Eq)>,
    pub name: LitStr,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct EntryArgInclude {
    pub include_token: Ident,
    pub paren_token: Paren,
    pub includes: Punctuated<LitStr, Comma>,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct EntryArgInline {
    pub token: Ident,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct EntryArgNoInline {
    pub token: Ident,
}

impl EntryArgs {
    pub fn try_to_entry(&self, item: &Item) -> syn::Result<Entry> {
        let default_name = item.get_default_name();
        let mut entry = Entry::default();
        let mut name = None;
        let mut inline = None;
        for arg in self.args.iter() {
            match arg {
                EntryArg::Name(arg) => {
                    if name.is_some() {
                        return Err(Error::new_spanned(arg, "duplicate `name` specified"));
                    }
                    name = Some(arg.name.value());
                }
                EntryArg::Include(arg) => {
                    entry
                        .include
                        .extend(arg.includes.iter().map(|lit| lit.value()));
                }
                EntryArg::Inline(arg) => {
                    if !item.is_mod() {
                        return Err(Error::new_spanned(arg, "expected to apply to `Module`"));
                    }
                    if let Some(inline) = inline {
                        return Err(Error::new_spanned(
                            arg,
                            if inline {
                                "duplicate `inline` specified"
                            } else {
                                "already `no_inline` specified"
                            },
                        ));
                    }
                    inline = Some(true);
                }
                EntryArg::NoInline(arg) => {
                    if !item.is_mod() {
                        return Err(Error::new_spanned(arg, "expected to apply to `Module`"));
                    }
                    if let Some(inline) = inline {
                        return Err(Error::new_spanned(
                            arg,
                            if inline {
                                "already `inline` specified"
                            } else {
                                "duplicate `no_inline` specified"
                            },
                        ));
                    }
                    inline = Some(false);
                }
            }
        }
        if let Some(inline) = inline {
            entry.inline = inline;
        }
        if name.is_none() {
            name = default_name;
        }
        if let Some(name) = name {
            entry.name = name;
        } else {
            return Err(Error::new_spanned(self, "`name` unspecified"));
        }
        Ok(entry)
    }
}

impl Parse for EntryArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            args: input.parse_terminated(EntryArg::parse)?,
        })
    }
}

impl Parse for EntryArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            input.parse().map(Self::Name)
        } else if lookahead.peek(Ident::peek_any) {
            let token: Ident = input.parse()?;
            match token.to_string().as_str() {
                "name" => EntryArgName::parse_after_token(token, input).map(Self::Name),
                "include" => EntryArgInclude::parse_after_token(token, input).map(Self::Include),
                "inline" => EntryArgInline::parse_after_token(token, input).map(Self::Inline),
                "no_inline" => {
                    EntryArgNoInline::parse_after_token(token, input).map(Self::NoInline)
                }
                _ => Err(input.error("expected `name` | `include` | `inline` | `no_inline`")),
            }
        } else {
            Err(input.error("expected `name` | `include` | `inline` | `no_inline`"))
        }
    }
}

impl EntryArgName {
    fn parse_after_token(token: Ident, input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name_token: Some((token, input.parse()?)),
            name: input.parse()?,
        })
    }
}

impl Parse for EntryArgName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name_token: None,
            name: input.parse()?,
        })
    }
}

#[allow(clippy::eval_order_dependence)]
impl EntryArgInclude {
    fn parse_after_token(include_token: Ident, input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            include_token,
            paren_token: parenthesized!(content in input),
            includes: content.call(Punctuated::parse_separated_nonempty)?,
        })
    }
}

impl EntryArgInline {
    fn parse_after_token(token: Ident, _input: ParseStream) -> syn::Result<Self> {
        Ok(Self { token })
    }
}

impl EntryArgNoInline {
    fn parse_after_token(token: Ident, _input: ParseStream) -> syn::Result<Self> {
        Ok(Self { token })
    }
}

impl ToTokens for EntryArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.args.to_tokens(tokens);
    }
}

impl ToTokens for EntryArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            EntryArg::Name(arg) => arg.to_tokens(tokens),
            EntryArg::Include(arg) => arg.to_tokens(tokens),
            EntryArg::Inline(arg) => arg.to_tokens(tokens),
            EntryArg::NoInline(arg) => arg.to_tokens(tokens),
        }
    }
}

impl ToTokens for EntryArgName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some((name_token, eq)) = &self.name_token {
            name_token.to_tokens(tokens);
            eq.to_tokens(tokens);
        }
        self.name.to_tokens(tokens);
    }
}

impl ToTokens for EntryArgInclude {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.include_token.to_tokens(tokens);
        self.paren_token
            .surround(tokens, |tokens| self.includes.to_tokens(tokens));
    }
}

impl ToTokens for EntryArgInline {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.token.to_tokens(tokens)
    }
}

impl ToTokens for EntryArgNoInline {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.token.to_tokens(tokens)
    }
}
