use crate::field::attrs::Attrs;
use quote::quote;

pub struct KeyValueFieldOptions {
    /// What to do when the key is missing or invalid.
    dne_strategy: DneStrategy,

    /// Additional constants for what constitutes a 'valid' value (beyond being able to parse it).
    ///
    /// This should be an expression to a closure of the form: `fn(&T) -> Result<(), E: Display>`.
    validator: Option<syn::Expr>,
}

pub enum DneStrategy {
    /// The key must exist and be valid.
    Require,

    /// The key is allowed to be missing or invalid.
    Optional,

    /// The key will be replaced with a default value if missing or invalid.
    Default(syn::Expr),
}

impl KeyValueFieldOptions {
    pub fn from_attrs(attrs: Attrs, is_optional: bool) -> syn::Result<Self> {
        Self::assert_only_kv_attrs(&attrs)?;

        let dne_strategy = match attrs.default {
            Some(attr) if is_optional => {
                return Err(syn::Error::new(
                    attr.span,
                    "default value not allowed with optional values",
                ));
            }
            Some(attr) => DneStrategy::Default(attr.expr),
            None if is_optional => DneStrategy::Optional,
            _ => DneStrategy::Require,
        };

        Ok(Self {
            dne_strategy,
            validator: attrs.validator.map(|v| v.expr),
        })
    }

    fn assert_only_kv_attrs(attrs: &Attrs) -> syn::Result<()> {
        if let Some(attr) = &attrs.required_if {
            return Err(syn::Error::new(
                attr.span,
                "invalid attribute `required_if` on key value field",
            ));
        }
        Ok(())
    }

    pub fn get_parse_expr_tokens(&self, field_name: &syn::Ident) -> proc_macro2::TokenStream {
        let key = field_name.to_string().to_uppercase();
        match &self.dne_strategy {
            DneStrategy::Require => self.non_default_parse_tokens(key, false),
            DneStrategy::Optional => self.non_default_parse_tokens(key, true),
            DneStrategy::Default(default) => match &self.validator {
                Some(validator) => {
                    quote! {
                        ctx.parse_validated_with_default(#key, #validator, || (#default).into())
                    }
                }
                None => {
                    quote! {
                        ctx.parse_with_default(#key, || (#default).into())
                    }
                }
            },
        }
    }

    fn non_default_parse_tokens(&self, key: String, optional: bool) -> proc_macro2::TokenStream {
        match &self.validator {
            Some(validator) => {
                quote! {
                    ctx.parse_validated(#key, #validator, #optional)
                }
            }
            None => {
                quote! {
                    ctx.parse(#key, #optional)
                }
            }
        }
    }

    pub fn needs_unwrap(&self) -> bool {
        matches!(self.dne_strategy, DneStrategy::Require)
    }

    pub fn is_optional(&self) -> bool {
        matches!(self.dne_strategy, DneStrategy::Optional)
    }
}
