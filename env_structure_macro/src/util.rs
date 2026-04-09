/// Checks if a type is optional and returns the type stripped of the option.
///
/// The boolean indicates whether the type is optional (`true` means optional).
pub fn extract_optional_type(ty: syn::Type) -> (syn::Type, bool) {
    // Optionals are found in path types.
    let syn::Type::Path(path) = &ty else {
        return (ty, false);
    };

    // See if the path leads to an option.
    let last = match path.path.segments.last() {
        Some(seg) if seg.ident == "Option" => seg,
        _ => return (ty, false),
    };

    // Get the generic arguments (the inner type is located inside).
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return (ty, false);
    };

    // Check if the first argument exists and is generic.
    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
        return (inner.clone(), true);
    }

    (ty, false)
}
