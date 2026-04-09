use syn::Token;
use syn::parse::{Parse, ParseStream};

pub struct Attrs {
    pub nested: bool,
    pub default: Option<DefaultAttr>,
    pub validator: Option<ValidatorAttr>,
    pub required_if: Option<RequiredIfAttr>,
}

pub struct DefaultAttr {
    pub span: proc_macro2::Span,
    pub expr: syn::Expr,
}

pub struct ValidatorAttr {
    pub span: proc_macro2::Span,
    pub expr: syn::Expr,
}

pub struct RequiredIfAttr {
    pub span: proc_macro2::Span,
    pub key: syn::LitStr,
}

impl Attrs {
    fn empty() -> Self {
        Self {
            nested: false,
            default: None,
            validator: None,
            required_if: None,
        }
    }

    pub fn parse_all(raw_attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut parsed_attrs = Self::empty();
        for attr in raw_attrs {
            // Skip non-recognized attributes.
            if !attr.path().is_ident("env") {
                continue;
            }
            let attrs: Self = attr.parse_args()?;
            parsed_attrs = parsed_attrs.merge_all(attrs)?;
        }
        Ok(parsed_attrs)
    }

    fn merge_all(mut self, other: Self) -> syn::Result<Self> {
        self.nested |= other.nested;
        if let Some(val) = other.default {
            self.merge_default(val)?;
        }
        if let Some(val) = other.validator {
            self.merge_validator(val)?;
        }
        if let Some(val) = other.required_if {
            self.merge_required_if(val)?;
        }
        Ok(self)
    }

    fn merge_default(&mut self, val: DefaultAttr) -> syn::Result<()> {
        if self.default.is_some() {
            return Err(syn::Error::new(val.span, "duplicate `default` attribute"));
        }
        self.default = Some(val);
        Ok(())
    }

    fn merge_validator(&mut self, val: ValidatorAttr) -> syn::Result<()> {
        if self.validator.is_some() {
            return Err(syn::Error::new(val.span, "duplicate `validator` attribute"));
        }
        self.validator = Some(val);
        Ok(())
    }

    fn merge_required_if(&mut self, val: RequiredIfAttr) -> syn::Result<()> {
        if self.required_if.is_some() {
            return Err(syn::Error::new(
                val.span,
                "duplicate `required_if` attribute",
            ));
        }
        self.required_if = Some(val);
        Ok(())
    }
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = Self::empty();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                // Nested is a simple flag.
                "nested" => attrs.nested = true,

                // Non-flag attributes.
                "default" => {
                    input.parse::<Token![=]>()?;
                    let expr = input.parse()?;
                    attrs.merge_default(DefaultAttr {
                        span: ident.span(),
                        expr,
                    })?;
                }
                "validator" => {
                    input.parse::<Token![=]>()?;
                    let expr = input.parse()?;
                    attrs.merge_validator(ValidatorAttr {
                        span: ident.span(),
                        expr,
                    })?;
                }
                "required_if" => {
                    let content;
                    syn::parenthesized!(content in input);
                    let key = content.parse()?;
                    attrs.merge_required_if(RequiredIfAttr {
                        span: ident.span(),
                        key,
                    })?;
                }

                // Unrecognized.
                _ => {
                    return Err(syn::Error::new_spanned(&ident, "unrecognized argument"));
                }
            }

            // Consume the separating comma, if present
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(attrs)
    }
}
