use crate::field::options::FieldOptions;
use crate::util::extract_optional_type;

mod attrs;
mod key_value;
mod nested;
mod options;

/// Data describing how a field should load from its environment.
pub struct Field {
    /// The name of the field.
    name: syn::Ident,

    /// The non-optional datatype of the field.
    concrete_type: syn::Type,

    /// The specific options related to how to get field data (and what the field represents).
    options: FieldOptions,
}

impl Field {
    /// Parses the [`Field`] from a [`syn::Field`].
    pub fn parse(field: syn::Field) -> syn::Result<Self> {
        // Extract the field name.
        let Some(name) = field.ident else {
            return Err(syn::Error::new_spanned(field, "field must have a name"));
        };

        // Extract the concrete type.
        let (concrete_type, is_optional) = extract_optional_type(field.ty);

        // Parse the field arguments to determine its options.
        let options = FieldOptions::parse(field.attrs, name.span(), is_optional)?;

        Ok(Self {
            name,
            concrete_type,
            options,
        })
    }

    pub fn get_parse_binding_tokens(&self) -> proc_macro2::TokenStream {
        let binding = &self.name;
        let parse_expr = self.options.get_parse_expr_tokens(&self.name);
        quote::quote! {
            let #binding = #parse_expr;
        }
    }

    pub fn get_struct_construction_field_tokens(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        if self.options.needs_unwrap() {
            quote::quote! {
                #name: #name.unwrap()
            }
        } else {
            quote::quote! {
                #name
            }
        }
    }
}
