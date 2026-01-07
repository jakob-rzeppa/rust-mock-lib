use quote::quote;
use syn::{FnArg, Type, TypeReference};
use syn::punctuated::Punctuated;
use syn::token::Comma;

/// Creates a type representation for function parameters.
///
/// Converts a list of function parameters into a single type that can be used
/// as a generic parameter for the mock infrastructure.
///
/// # Returns
///
/// - For 0 parameters: `()`
/// - For 1 parameter: The parameter type itself
/// - For 2+ parameters: A tuple of all parameter types
///
/// # Examples
///
/// - `fn foo()` → `()`
/// - `fn foo(x: i32)` → `i32`
/// - `fn foo(x: i32, y: String)` → `(i32, String)`
///
/// # Panics
///
/// Panics if the function has a `self` parameter, as methods cannot be mocked.
pub(crate) fn create_param_type(fn_inputs: &Punctuated<FnArg, Comma>) -> Type {
    let param_types: Vec<_> = fn_inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(&pat_type.ty),
            syn::FnArg::Receiver(_) => panic!(
                "mock_function does not support methods with 'self' parameters. \
                 Only standalone functions can be mocked."
            ),
        })
        .collect();

    // Single parameter doesn't need tuple wrapping
    if param_types.len() == 1 {
        param_types[0].as_ref().clone()
    } else {
        // Multiple parameters or no parameters use tuple syntax
        syn::parse2(quote! { (#(#param_types),*) }).unwrap()
    }
}

/// Creates a tuple expression from function parameter names.
///
/// Converts parameter patterns into a tuple that can be passed to the mock
/// implementation for call tracking and verification.
///
/// # Returns
///
/// - For 0 parameters: `()`
/// - For 1 parameter: The parameter name itself (not wrapped in tuple)
/// - For 2+ parameters: A tuple of all parameter names
///
/// # Examples
///
/// - `fn foo()` → `()`
/// - `fn foo(x: i32)` → `x`
/// - `fn foo(x: i32, y: String)` → `(x, y)`
///
/// # Panics
///
/// Panics if the function has a `self` parameter, as methods cannot be mocked.
pub(crate) fn create_tuple_from_param_names(fn_inputs: &Punctuated<FnArg, Comma>) -> proc_macro2::TokenStream {
    let param_names: Vec<_> = fn_inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(&pat_type.pat),
            syn::FnArg::Receiver(_) => panic!(
                "mock_function does not support methods with 'self' parameters"
            ),
        })
        .collect();

    if param_names.is_empty() {
        quote! { () }
    } else if param_names.len() == 1 {
        let name = &param_names[0];
        quote! { #name }
    } else {
        quote! { (#(#param_names),*) }
    }
}

/// Checks if a type contains references (fails the 'static bound).
///
/// Returns true if the type is a reference or contains references that would
/// prevent it from satisfying the 'static lifetime bound.
fn contains_reference(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) => true,
        Type::Tuple(tuple) => tuple.elems.iter().any(|t| contains_reference(t)),
        Type::Array(arr) => contains_reference(&arr.elem),
        Type::Slice(slice) => contains_reference(&slice.elem),
        Type::Paren(paren) => contains_reference(&paren.elem),
        Type::Group(group) => contains_reference(&group.elem),
        _ => false, // Other types are assumed to be 'static unless they contain references
    }
}

/// Validates that all function parameters satisfy the 'static bound.
///
/// Returns an error if any parameter contains references, as the mock infrastructure
/// requires all parameters to be 'static (no borrowed data).
///
/// # Returns
///
/// - `Ok(())` if all parameters are 'static
/// - `Err(syn::Error)` if any parameter contains references
pub(crate) fn validate_static_params(fn_inputs: &Punctuated<FnArg, Comma>) -> syn::Result<()> {
    for arg in fn_inputs.iter() {
        if let FnArg::Typed(pat_type) = arg {
            if contains_reference(&pat_type.ty) {
                return Err(syn::Error::new_spanned(
                    &pat_type.ty,
                    "mock_function requires all parameters to be 'static. \
                     Parameters cannot contain references. \
                     Consider using owned types like String instead of &str, \
                     or Vec<T> instead of &[T]."
                ));
            }
        }
    }
    Ok(())
}
