use crate::field::attrs::Attrs;
use quote::quote;

pub struct NestedFieldOptions {
    /// The condition to be met for the nested object to be parsed.
    ///
    /// If set, the field must be optional; otherwise, it must be required.
    condition: Option<Condition>,
}

pub struct Condition {
    /// The condition key.
    key: syn::LitStr,
    // TODO: value: syn::Expr,
}

impl NestedFieldOptions {
    pub fn from_attrs(
        attrs: Attrs,
        span: proc_macro2::Span,
        is_optional: bool,
    ) -> syn::Result<Self> {
        Self::assert_only_nested_attrs(&attrs)?;

        let condition = match attrs.required_if {
            None if is_optional => {
                return Err(syn::Error::new(
                    span,
                    "attribute `required_if` is required for optional nested fields",
                ));
            }
            Some(attr) if !is_optional => {
                return Err(syn::Error::new(
                    attr.span,
                    "attribute `required_if` can only be applied to optional nested fields",
                ));
            }
            attr => attr.map(|v| Condition { key: v.key }),
        };

        Ok(Self { condition })
    }

    fn assert_only_nested_attrs(attrs: &Attrs) -> syn::Result<()> {
        if let Some(attr) = &attrs.default {
            return Err(syn::Error::new(
                attr.span,
                "invalid attribute `default` on nested field",
            ));
        }
        if let Some(attr) = &attrs.validator {
            return Err(syn::Error::new(
                attr.span,
                "invalid attribute `validator` on nested field",
            ));
        }
        Ok(())
    }

    pub fn get_parse_expr_tokens(&self) -> proc_macro2::TokenStream {
        match &self.condition {
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
        }
    }

    pub fn needs_unwrap(&self) -> bool {
        self.condition.is_none()
    }

    pub fn is_optional(&self) -> bool {
        self.condition.is_some()
    }
}
