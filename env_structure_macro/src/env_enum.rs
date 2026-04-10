use crate::util::make_env_struct_impl;
use crate::variant::Variant;
use proc_macro2::Ident;
use std::collections::HashSet;
use syn::Token;
use syn::parse::ParseStream;

pub struct EnvEnum {
    name: syn::Ident,
    key: syn::LitStr,
    default: Option<syn::Ident>,
    variants: Vec<Variant>,
}

impl EnvEnum {
    pub fn parse(
        name: syn::Ident,
        attrs: Vec<syn::Attribute>,
        data: syn::DataEnum,
    ) -> syn::Result<Self> {
        // Get the enum key attribute.
        let key = get_key_attr(&name, attrs)?;

        // Get variants
        let mut variants = Vec::with_capacity(data.variants.len());
        for variant in data.variants {
            variants.push(Variant::parse(variant)?);
        }

        // Validate default and values
        let default = validate_variants_extract_default(variants.as_ref())?;

        // Return
        Ok(Self {
            name,
            key,
            default,
            variants,
        })
    }

    pub fn to_env_struct_impl(self) -> proc_macro2::TokenStream {
        let key_enum = self.key_enum_tokens();
        let get_key = self.get_key_tokens();
        let match_variants = self.variants.into_iter().map(|v| v.to_parse_match_tokens());

        let parse_inner = quote::quote! {
            #key_enum
            #get_key
            match key {
                #(#match_variants),*
            }
        };

        make_env_struct_impl(self.name, parse_inner)
    }

    fn key_enum_tokens(&self) -> proc_macro2::TokenStream {
        let variants = self.variants.iter().map(|v| &v.name);
        let parse_variants = self.variants.iter().map(|v| {
            let value = &v.value;
            let variant = &v.name;
            quote::quote! {
                #value => ::std::result::Result::Ok(__Key::#variant)
            }
        });
        let discriminant_error = format!("not a valid discriminant for {}", self.name.to_string());
        let display_variants = self.variants.iter().map(|v| {
            let value = &v.value;
            let variant = &v.name;
            quote::quote! {
                __Key::#variant => <str as std::fmt::Display>::fmt(#value, f)
            }
        });
        quote::quote! {
            enum __Key {
                #(#variants),*
            }
            impl ::env_structure::FromEnv for __Key {
                fn parse(input: ::std::result::Result<::std::string::String, ::std::env::VarError>) -> ::std::result::Result<Self, ::env_structure::ParseIssueKind> {
                    let input = input?;
                    match input.as_str() {
                        #(#parse_variants,)*
                        _ => std::result::Result::Err(::env_structure::ParseIssueKind::InvalidValue {
                            value: input,
                            msg: #discriminant_error.to_string()
                        })
                    }
                }
            }
            impl ::env_structure::EnvDisplay for __Key {
                fn display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#display_variants),*
                    }
                }
            }
        }
    }

    fn get_key_tokens(&self) -> proc_macro2::TokenStream {
        let key_lit = &self.key;
        match &self.default {
            Some(variant) => {
                quote::quote! {
                    let key = ctx.parse_with_default(#key_lit, || __Key::#variant);
                }
            }
            None => {
                quote::quote! {
                    let ::std::option::Option::Some(key) = ctx.parse(#key_lit, false) else {
                        return ::std::option::Option::None;
                    }
                }
            }
        }
    }
}

fn get_key_attr(name: &syn::Ident, attrs: Vec<syn::Attribute>) -> syn::Result<syn::LitStr> {
    let mut key = None;
    for attr in attrs {
        // Skip non-recognized attributes.
        if !attr.path().is_ident("env") {
            continue;
        }

        // Extract the key.
        let res =
            attr.parse_args_with(|input: ParseStream| -> syn::Result<Option<syn::LitStr>> {
                let mut key = None;
                while !input.is_empty() {
                    let ident: syn::Ident = input.parse()?;
                    match ident.to_string().as_str() {
                        "key" => {
                            input.parse::<Token![=]>()?;
                            let val = input.parse()?;
                            if key.is_some() {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "duplicate `key` attribute",
                                ));
                            }
                            key = Some(val);
                        }
                        _ => return Err(syn::Error::new_spanned(ident, "unrecognized argument")),
                    }
                }
                Ok(key)
            })?;

        if let Some(res) = res {
            if key.is_some() {
                return Err(syn::Error::new_spanned(res, "duplicate `key` attribute"));
            }
            key = Some(res);
        }
    }

    key.ok_or_else(|| syn::Error::new_spanned(name, "missing required `key` attribute"))
}

fn validate_variants_extract_default(variants: &[Variant]) -> syn::Result<Option<syn::Ident>> {
    let mut enum_default = None;
    let mut values = HashSet::with_capacity(variants.len());
    for variant in variants {
        // Make sure there is only one default variant.
        if let Some(default) = &variant.default
            && enum_default.is_some()
        {
            return Err(syn::Error::new_spanned(
                default,
                "there can only be one default variant",
            ));
        }
        if variant.default.is_some() {
            enum_default = Some(variant.name.clone());
        }

        // Make sure all variant values are unique.
        let value = variant.value.value();
        let fresh_value = values.insert(value);
        if !fresh_value {
            return Err(syn::Error::new_spanned(
                &variant.value,
                "duplicate variant value",
            ));
        }
    }
    Ok(enum_default)
}
