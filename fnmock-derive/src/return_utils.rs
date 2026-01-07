use quote::quote;

/// Extracts the return type from a function signature.
///
/// Converts a `ReturnType` into a concrete `Type` that can be used
/// as a generic parameter for the mock infrastructure.
///
/// # Returns
///
/// - For no return type (default): `()`
/// - For explicit return type: The specified type
///
/// # Examples
///
/// - `fn foo()` → `()`
/// - `fn foo() -> String` → `String`
/// - `fn foo() -> Result<(), Error>` → `Result<(), Error>`
pub(crate) fn extract_return_type(return_type: &syn::ReturnType) -> syn::Type {
    match return_type {
        syn::ReturnType::Default => syn::parse2(quote! { () }).unwrap(),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    }
}
