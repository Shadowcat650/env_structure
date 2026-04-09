use crate::field::attrs::Attrs;
use crate::field::key_value::KeyValueFieldOptions;
use crate::field::nested::NestedFieldOptions;

pub enum FieldOptions {
    /// How to load a key from the environment.
    KeyValue(KeyValueFieldOptions),

    /// How to load a nested environment structure.
    Nested(NestedFieldOptions),
}

impl FieldOptions {
    pub fn parse(
        field_attrs: Vec<syn::Attribute>,
        span: proc_macro2::Span,
        is_optional: bool,
    ) -> syn::Result<Self> {
        let attrs = Attrs::parse_all(field_attrs)?;
        if attrs.nested {
            Ok(Self::Nested(NestedFieldOptions::from_attrs(
                attrs,
                span,
                is_optional,
            )?))
        } else {
            Ok(Self::KeyValue(KeyValueFieldOptions::from_attrs(
                attrs,
                is_optional,
            )?))
        }
    }

    pub fn get_parse_expr_tokens(&self, field_name: &syn::Ident) -> proc_macro2::TokenStream {
        match self {
            FieldOptions::KeyValue(opts) => opts.get_parse_expr_tokens(field_name),
            FieldOptions::Nested(opts) => opts.get_parse_expr_tokens(),
        }
    }

    pub fn needs_unwrap(&self) -> bool {
        match self {
            FieldOptions::KeyValue(opts) => opts.needs_unwrap(),
            FieldOptions::Nested(opts) => opts.needs_unwrap(),
        }
    }
}
