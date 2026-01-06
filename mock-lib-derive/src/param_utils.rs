//! Utility functions for processing function parameters in mock generation.
//!
//! This module provides helper functions to transform function parameters into
//! types and expressions suitable for the mock infrastructure.

use quote::quote;
use syn::{FnArg, Type};
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
