mod env_enum;
mod env_struct;
mod field;
mod util;
mod variant;

use crate::env_enum::EnvEnum;
use crate::env_struct::EnvStruct;

#[proc_macro_derive(EnvStructure, attributes(env))]
pub fn env_structure(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match expand_input(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn expand_input(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match input.data {
        syn::Data::Struct(data) => {
            let env_struct = EnvStruct::parse(input.ident, data)?;
            Ok(env_struct.to_env_struct_impl())
        }
        syn::Data::Enum(data) => {
            let env_enum = EnvEnum::parse(input.ident, input.attrs, data)?;
            Ok(env_enum.to_env_struct_impl())
        }
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "EnvStructure does not support unions",
        )),
    }
}
