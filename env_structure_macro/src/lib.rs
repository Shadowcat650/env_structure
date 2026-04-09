mod env_struct;
mod field;
mod util;

use crate::env_struct::EnvStruct;

#[proc_macro_derive(EnvStructure, attributes(env))]
pub fn env_structure(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match derive_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn derive_inner(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let syn::Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "EnvStructure only supports structs",
        ));
    };
    let env_struct = EnvStruct::parse(input.ident, data)?;
    Ok(env_struct.to_impl())
}
