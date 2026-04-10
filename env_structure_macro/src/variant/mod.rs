use crate::env_struct::EnvStruct;
use crate::field::Field;
use crate::variant::attrs::Attrs;

mod attrs;

pub struct Variant {
    pub name: syn::Ident,
    pub value: syn::LitStr,
    pub default: Option<syn::Ident>,
    content: VariantContent,
}

impl Variant {
    pub fn parse(variant: syn::Variant) -> syn::Result<Self> {
        let name = variant.ident;

        // Parse the variant attributes.
        let attrs = Attrs::parse_all(variant.attrs)?;
        let Some(value) = attrs.value else {
            return Err(syn::Error::new_spanned(
                name,
                "missing required `value` attribute",
            ));
        };

        // Parse the contents.
        let content = VariantContent::parse(variant.fields)?;

        Ok(Self {
            name,
            value: value.value,
            default: attrs.default,
            content,
        })
    }

    pub fn to_parse_match_tokens(self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let inner = match self.content {
            VariantContent::Unit => quote::quote! {
                ::std::option::Option::Some(Self::#name)
            },
            VariantContent::Nested(_ty) => quote::quote! {
                ctx.parse_nested().map(Self::#name)
            },
            VariantContent::InlineStruct(x) => inline_struct_match_tokens_inner(name, x),
        };
        quote::quote! {
            __Key::#name => { #inner }
        }
    }
}

fn inline_struct_match_tokens_inner(
    name: &syn::Ident,
    fields: Vec<Field>,
) -> proc_macro2::TokenStream {
    let self_fields = fields.iter().map(|f| {
        let name = &f.name;
        quote::quote! {
            #name: x.#name
        }
    });
    let self_tokens = quote::quote! {
        Self::#name {
            #(#self_fields),*
        }
    };

    let inner_struct_name = syn::Ident::new("__Inner", proc_macro2::Span::call_site());
    let inner_struct_fields = fields.iter().map(|f| {
        let name = &f.name;
        let ty_concrete = &f.concrete_type;
        let ty = if f.is_optional() {
            quote::quote! { ::std::option::Option<#ty_concrete> }
        } else {
            quote::quote! { #ty_concrete }
        };
        quote::quote! {
            #name: #ty
        }
    });
    let inner_struct_def = quote::quote! {
        struct #inner_struct_name {
            #(#inner_struct_fields),*
        }
    };
    let inner_struct_impl = EnvStruct {
        name: inner_struct_name,
        fields,
    }
    .to_env_struct_impl();

    quote::quote! {
        #inner_struct_def
        #inner_struct_impl
        ctx.parse_nested().map(|x: __Inner| #self_tokens)
    }
}

enum VariantContent {
    Unit,
    Nested(syn::Type),
    InlineStruct(Vec<Field>),
}

impl VariantContent {
    pub fn parse(fields: syn::Fields) -> syn::Result<Self> {
        let mut iter = fields.into_iter();
        let Some(first) = iter.next() else {
            return Ok(Self::Unit);
        };

        if iter.len() == 0 && first.ident.is_none() {
            return Ok(Self::Nested(first.ty));
        }

        let mut fields = Vec::with_capacity(iter.len() + 1);
        for field in std::iter::once(first).chain(iter) {
            fields.push(Field::parse(field)?);
        }

        Ok(Self::InlineStruct(fields))
    }
}
