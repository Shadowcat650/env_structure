use crate::field::Field;

pub struct EnvStruct {
    name: syn::Ident,
    fields: Vec<Field>,
}

impl EnvStruct {
    pub fn parse(name: syn::Ident, data: syn::DataStruct) -> syn::Result<Self> {
        let mut fields = Vec::with_capacity(data.fields.len());
        for field in data.fields {
            fields.push(Field::parse(field)?);
        }
        Ok(Self { name, fields })
    }

    pub fn to_impl(self) -> proc_macro2::TokenStream {
        let name = self.name;
        let bindings = self.fields.iter().map(|f| f.get_parse_binding_tokens());
        let stores = self
            .fields
            .iter()
            .map(|f| f.get_struct_construction_field_tokens());
        quote::quote! {
            impl ::env_structure::EnvStructure for #name {
                fn parse(ctx: &mut ::env_structure::ParseCtx) -> ::std::option::Option<Self> {
                    #(#bindings)*
                    if ctx.has_errors() {
                        return ::std::option::Option::None;
                    }
                    ::std::option::Option::Some(Self {
                        #(#stores),*
                    })
                }
            }
        }
    }
}
