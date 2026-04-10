use syn::Token;
use syn::parse::{Parse, ParseStream};

pub struct Attrs {
    /// If `Some`, the attribute is marked as default.
    pub default: Option<syn::Ident>,
    pub value: Option<Value>,
}

pub struct Value {
    span: proc_macro2::Span,
    pub value: syn::LitStr,
}

impl Attrs {
    fn empty() -> Self {
        Self {
            default: None,
            value: None,
        }
    }

    pub fn parse_all(raw_attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut parsed_attrs = Self::empty();
        for attr in raw_attrs {
            // Skip non-recognized attributes.
            if !attr.path().is_ident("env") {
                continue;
            }
            let attrs = attr.parse_args()?;
            parsed_attrs = parsed_attrs.merge(attrs)?;
        }
        Ok(parsed_attrs)
    }

    fn merge(mut self, other: Self) -> syn::Result<Self> {
        if let Some(val) = other.default {
            self.marge_default(val)?;
        }
        if let Some(val) = other.value {
            self.merge_value(val)?;
        }
        Ok(self)
    }

    fn marge_default(&mut self, val: syn::Ident) -> syn::Result<()> {
        if self.default.is_some() {
            return Err(syn::Error::new_spanned(
                &val,
                "duplicate `default` attribute",
            ));
        }
        self.default = Some(val);
        Ok(())
    }

    fn merge_value(&mut self, val: Value) -> syn::Result<()> {
        if self.value.is_some() {
            return Err(syn::Error::new(val.span, "duplicate `value` attribute"));
        }
        self.value = Some(val);
        Ok(())
    }
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = Self::empty();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "default" => attrs.marge_default(ident)?,
                "value" => {
                    input.parse::<Token![=]>()?;
                    let val = input.parse()?;
                    attrs.merge_value(Value {
                        span: ident.span(),
                        value: val,
                    })?;
                }

                // Unrecognized.
                _ => return Err(syn::Error::new_spanned(ident, "unrecognized argument")),
            }

            // Consume the separating comma, if present
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(attrs)
    }
}
