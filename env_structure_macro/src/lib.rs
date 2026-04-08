use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DataStruct, Token};

#[proc_macro_derive(EnvStructure, attributes(env))]
pub fn env_structure(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match derive_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn derive_inner(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "EnvStructure only supports structs",
        ));
    };

    let fields = parse_fields(data)?;

    let field_lets = fields.iter().map(|field| match &field.options {
        FieldOptions::Value(options) => {
            let uppercase = field.name.to_string().to_uppercase();
            let parse = match &options.dne_strategy {
                DneStrategy::Require | DneStrategy::Optional => {
                    let optional = match options.dne_strategy {
                        DneStrategy::Optional => true,
                        _ => false,
                    };
                    match &options.validator {
                        Some(validator) => {
                            quote! {
                                ctx.parse_validated(#uppercase, #validator, #optional)
                            }
                        }
                        None => {
                            quote! {
                                ctx.parse(#uppercase, #optional)
                            }
                        }
                    }
                }
                DneStrategy::Default(val) => {
                    let ty = &field.ty;
                    match &options.validator {
                        Some(validator) => {
                            quote! {
                                ctx.parse_validated_with_default(#uppercase, #validator, || -> #ty {(#val).into()})
                            }
                        }
                        None => {
                            quote! {
                                ctx.parse_with_default(#uppercase, || -> #ty {(#val).into()})
                            }
                        }
                    }
                },
            };
            let name = &field.name;

            quote! {
                let #name = #parse;
            }
        }
        FieldOptions::Nested(options) => {
            let parse = match &options.condition {
                Some(cond) => {
                    let key = &cond.key;
                    quote! {
                        ctx.parse_nested_if(#key)
                    }
                }
                None => {
                    quote! {
                        ctx.parse_nested()
                    }
                }
            };

            let name = &field.name;
            quote! {
                let #name = #parse;
            }
        }
    });
    let field_stores = fields.iter().map(|field| {
        let name = &field.name;
        if field.needs_unwrap {
            quote! {
                #name: #name.unwrap()
            }
        } else {
            quote! {
                #name
            }
        }
    });

    let name = input.ident;
    Ok(quote! {
        impl ::env_structure::EnvStructure for #name {
            fn parse(ctx: &mut ::env_structure::ParseCtx) -> ::std::option::Option<Self> {
                #(#field_lets)*
                if ctx.has_errors() {
                    return ::std::option::Option::None;
                }
                ::std::option::Option::Some(Self {
                    #(#field_stores),*
                })
            }
        }
    })
}

fn parse_fields(data: DataStruct) -> syn::Result<Vec<Field>> {
    let mut fields = Vec::with_capacity(data.fields.len());
    for field in data.fields {
        fields.push(Field::new(field)?);
    }
    Ok(fields)
}

struct Field {
    name: syn::Ident,
    ty: syn::Type,
    needs_unwrap: bool,
    options: FieldOptions,
}

impl Field {
    fn new(field: syn::Field) -> syn::Result<Self> {
        let Some(name) = field.ident else {
            return Err(syn::Error::new_spanned(
                &field,
                "field must have an identifier",
            ));
        };

        // Get all env arguments on the field.
        let mut field_args = Vec::new();
        for attr in field.attrs {
            if !attr.path().is_ident("env") {
                continue;
            }
            let args: EnvArgs = attr.parse_args()?;
            field_args.extend(args.0);
        }

        let (is_optional, ty) = match extract_option_inner(&field.ty) {
            None => (false, field.ty),
            Some(ty) => (true, ty.clone()),
        };
        let options = FieldOptions::new(&ty, is_optional, field_args)?;
        let needs_unwrap = match &options {
            FieldOptions::Value(options) => match options.dne_strategy {
                DneStrategy::Require => true,
                DneStrategy::Optional | DneStrategy::Default(_) => false,
            },
            FieldOptions::Nested(options) => options.condition.is_none(),
        };

        Ok(Self {
            name,
            ty,
            needs_unwrap,
            options,
        })
    }
}

enum FieldOptions {
    Value(ValueFieldOptions),
    Nested(NestedFieldOptions),
}

impl FieldOptions {
    fn new(ty: &syn::Type, is_optional: bool, args: Vec<EnvArg>) -> syn::Result<Self> {
        let mut nested = false;
        let mut default = None;
        let mut validator = None;
        let mut required_if = None;
        for arg in args {
            match arg {
                EnvArg::Nested => nested = true,
                EnvArg::Default(val) => {
                    if default.is_some() {
                        return Err(syn::Error::new_spanned(&val, "duplicate default value"));
                    }
                    default = Some(val);
                }
                EnvArg::Validator(val) => {
                    if validator.is_some() {
                        return Err(syn::Error::new_spanned(&val, "duplicate validator"));
                    }
                    validator = Some(val);
                }
                EnvArg::RequiredIf(key) => {
                    if required_if.is_some() {
                        return Err(syn::Error::new_spanned(
                            &key,
                            "duplicate required condition",
                        ));
                    }
                    required_if = Some(Condition { key });
                }
            }
        }
        if nested {
            if let Some(default) = default {
                return Err(syn::Error::new_spanned(
                    &default,
                    "nested objects cannot have default values",
                ));
            } else if let Some(validator) = validator {
                return Err(syn::Error::new_spanned(
                    &validator,
                    "nested objects cannot have validators",
                ));
            }
            if required_if.is_some() && !is_optional {
                return Err(syn::Error::new_spanned(
                    ty,
                    "conditional nested objects must be optional",
                ));
            } else if required_if.is_none() && is_optional {
                return Err(syn::Error::new_spanned(
                    ty,
                    "non-conditional nested objects must not be optional",
                ));
            }
            Ok(Self::Nested(NestedFieldOptions {
                condition: required_if,
            }))
        } else {
            if let Some(condition) = required_if {
                return Err(syn::Error::new_spanned(
                    &condition.key,
                    "fields cannot have required conditions",
                ));
            }
            if let Some(val) = &default
                && is_optional
            {
                return Err(syn::Error::new_spanned(
                    val,
                    "a field cannot be optional and have a default field",
                ));
            }
            Ok(Self::Value(ValueFieldOptions {
                dne_strategy: if is_optional {
                    DneStrategy::Optional
                } else if let Some(val) = default {
                    DneStrategy::Default(val)
                } else {
                    DneStrategy::Require
                },
                validator,
            }))
        }
    }
}

struct ValueFieldOptions {
    dne_strategy: DneStrategy,
    validator: Option<syn::Expr>,
}

enum DneStrategy {
    Require,
    Optional,
    Default(syn::Expr),
}

struct NestedFieldOptions {
    /// The condition to be met for the nested object to be parsed.
    ///
    /// If set, the field must be optional; otherwise, it must be required.
    condition: Option<Condition>,
}

struct Condition {
    key: syn::LitStr,
    // TODO: value: syn::Expr,
}

enum EnvArg {
    Nested,
    Default(syn::Expr),
    Validator(syn::Expr),
    RequiredIf(syn::LitStr /* TODO: syn::Expr */),
}

struct EnvArgs(Vec<EnvArg>);

impl Parse for EnvArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "nested" => args.push(EnvArg::Nested),
                "default" => {
                    input.parse::<Token![=]>()?;
                    let expr = input.parse()?;
                    args.push(EnvArg::Default(expr));
                }
                "validator" => {
                    input.parse::<Token![=]>()?;
                    let expr = input.parse()?;
                    args.push(EnvArg::Validator(expr));
                }
                "required_if" => {
                    let content;
                    syn::parenthesized!(content in input);
                    let key = content.parse()?;
                    // content.parse::<Token![,]>()?;
                    // let val = content.parse()?;
                    args.push(EnvArg::RequiredIf(key));
                }
                _ => {
                    return Err(syn::Error::new_spanned(&ident, "unrecognized argument"));
                }
            }

            // Consume the separating comma, if present
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(EnvArgs(args))
    }
}

fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };

    let last = path.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(ref args) = last.arguments else {
        return None;
    };

    let inner = args.args.first()?;
    let syn::GenericArgument::Type(ty) = inner else {
        return None;
    };

    Some(ty)
}
