use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Type, ItemUse};

mod param_utils;
mod use_tree_processor;

use param_utils::{create_param_type, create_tuple_from_param_names};
use use_tree_processor::process_use_tree;

/// Attribute macro that generates a mockable version of a function.
///
/// This macro preserves the original function and generates:
/// 1. A `<function_name>_mock` function that can be called in tests
/// 2. A `<function_name>_mock` module containing mock control methods
///
/// # Generated Mock Module Methods
///
/// - `mock_implementation(fn)` - Sets a custom implementation for the mock
/// - `clear_mock()` - Resets the mock to its default panic behavior
/// - `assert_times(n)` - Verifies the function was called exactly n times
/// - `assert_with(params)` - Verifies the function was called with specific parameters
///
/// # Requirements
///
/// - Function must not have `self` parameters (standalone functions only)
/// - Function parameters must implement `Clone`, `Debug`, and `PartialEq` (for assertions)
///
/// # Example
///
/// ```ignore
/// use mock_lib::derive::mock_function;
///
/// #[mock_function]
/// pub(crate) fn fetch_user(id: u32) -> Result<String, String> {
///     // Real implementation
///     Ok(format!("user_{}", id))
/// }
///
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///
///     #[test]
///     fn test_with_mock() {
///         // Set up mock behavior
///         fetch_user_mock::mock_implementation(|id| {
///             Ok(format!("mock_user_{}", id))
///         });
///
///         // Call the mock
///         let result = fetch_user_mock(42);
///
///         // Verify behavior
///         assert_eq!(result, Ok("mock_user_42".to_string()));
///         fetch_user_mock::assert_times(1);
///         fetch_user_mock::assert_with(42);
///
///         // Clean up
///         fetch_user_mock::clear_mock();
///     }
/// }
/// ```
///
/// # Note
///
/// The mock function and module use thread-local storage, so mocks are isolated
/// between tests but **not thread-safe** if the same function is mocked in parallel
/// test threads.
///
/// This means if you write a test that spawns multiple threads
/// and those threads all try to mock the same function simultaneously,
/// you could encounter undefined behavior.
/// The mock state is isolated between different test threads (good for test independence),
/// but not protected within a single test that uses multiple threads.
#[proc_macro_attribute]
pub fn mock_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Extract function details
    let fn_visibility = input.vis.clone();
    let fn_name = input.sig.ident.clone();
    let fn_inputs = input.sig.inputs.clone();
    let fn_output = input.sig.output.clone();
    let fn_block = input.block.clone();

    // Generate mock function name
    let mock_fn_name = syn::Ident::new(&format!("{}_mock", &fn_name), fn_name.span());

    let params_type = create_param_type(&fn_inputs);
    let params_to_tuple = create_tuple_from_param_names(&fn_inputs);
    
    // Extract return type from ReturnType
    let return_type: Type = match &input.sig.output {
        syn::ReturnType::Default => syn::parse2(quote! { () }).unwrap(),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    };

    // Generate both the original function and the mock module
    let expanded = quote! {
        #fn_visibility fn #fn_name(#fn_inputs) #fn_output #fn_block

        pub(crate) fn #mock_fn_name(#fn_inputs) #fn_output {
            #mock_fn_name::call(#params_to_tuple)
        }

        pub(crate) mod #mock_fn_name {
            type Params = #params_type;
            type Return = #return_type;
            const FUNCTION_NAME: &str = "#mock_fn_name";

            thread_local! {
                static MOCK: std::cell::RefCell<mock_lib::function_mock::FunctionMock<
                    Params,
                    Return,
                >> = std::cell::RefCell::new(mock_lib::function_mock::FunctionMock::new(FUNCTION_NAME));
            }

            pub(crate) fn call(params: Params) -> Return {
                MOCK.with(|mock| {
                    mock.borrow_mut().call(params)
                })
            }

            pub(crate) fn mock_implementation(new_f: fn(Params) -> Return) {
                MOCK.with(|mock| {
                    mock.borrow_mut().mock_implementation(new_f)
                })
            }

            pub(crate) fn clear_mock() {
                MOCK.with(|mock|{
                    mock.borrow_mut().clear_mock()
                })
            }

            pub(crate) fn assert_times(expected_num_of_calls: u32) {
                MOCK.with(|mock| {
                    mock.borrow().assert_times(expected_num_of_calls)
                })
            }

            pub(crate) fn assert_with(params: Params) {
                MOCK.with(|mock| {
                    mock.borrow().assert_with(params)
                })
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute macro that conditionally imports functions and their mock versions.
///
/// This macro transforms a use statement to import the original function in production
/// code and the mock version (with `_mock` suffix) aliased to the original name in test code.
///
/// # Requirements
///
/// - The imported functions must have corresponding `_mock` versions generated by
///   the `#[mock_function]` attribute macro
/// - Only works with simple path imports and grouped imports
/// - Does not support glob imports (`*`) or renamed imports (`as`)
///
/// # Supported Patterns
///
/// ## Single function import:
/// ```ignore
/// #[use_function_mock]
/// use module::function;
/// ```
/// Expands to:
/// ```ignore
/// #[cfg(not(test))]
/// use module::function;
/// #[cfg(test)]
/// use module::function_mock as function;
/// ```
///
/// ## Multiple function imports:
/// ```ignore
/// #[use_function_mock]
/// use crate::service::{fetch_user, send_email};
/// ```
/// Expands to:
/// ```ignore
/// #[cfg(not(test))]
/// use crate::service::{fetch_user, send_email};
/// #[cfg(test)]
/// use crate::service::{fetch_user_mock as fetch_user, send_email_mock as send_email};
/// ```
#[proc_macro_attribute]
pub fn use_function_mock(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemUse);
    
    // Extract the module path and function name mappings
    let mut base_path = Vec::new();
    let function_mappings = process_use_tree(&input.tree, &mut base_path);
    
    // Reconstruct the module path as tokens
    let module_path = if base_path.is_empty() {
        quote! {}
    } else {
        quote! { #(#base_path)::* }
    };
    
    // Generate the appropriate expansion based on number of imports
    if function_mappings.len() == 1 {
        // Single import: use path::function;
        let (fn_name, mock_fn_name) = &function_mappings[0];
        let expanded = quote! {
            #[cfg(not(test))]
            #input
            
            #[cfg(test)]
            use #module_path::#mock_fn_name as #fn_name;
        };
        TokenStream::from(expanded)
    } else {
        // Multiple imports: use path::{fn1, fn2};
        let mock_alias_mappings: Vec<_> = function_mappings
            .iter()
            .map(|(fn_name, mock_fn_name)| {
                quote! { #mock_fn_name as #fn_name }
            })
            .collect();
        
        let expanded = quote! {
            #[cfg(not(test))]
            #input
            
            #[cfg(test)]
            use #module_path::{#(#mock_alias_mappings),*};
        };
        TokenStream::from(expanded)
    }
}
