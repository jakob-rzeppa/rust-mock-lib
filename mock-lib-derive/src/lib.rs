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
            const FUNCTION_NAME: &str = stringify!(#mock_fn_name);

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

/// Function-like procedural macro that conditionally selects between a function and its mock version inline.
///
/// This macro takes a function identifier and returns a block expression that evaluates to either
/// the original function in production builds or the mock version (with `_mock` suffix) in test builds.
/// The returned function can be immediately invoked with arguments using the syntax:
/// `use_inline_mock!(function_name)(args...)`
///
/// # Use Cases
///
/// In general, it is preferable to use the `#[use_function_mock]` attribute macro,
/// since it doesn't need to be placed in the code itself and mocks
/// all calls to the function in the module.
///
/// This macro should be used when:
/// - **You want to mock a function in the same module**
/// - You want to mock function calls inline without modifying import statements
/// - You need fine-grained control over which specific call sites use mocks vs. real implementations
///
/// # Requirements
///
/// - The function must have a corresponding `_mock` version generated by the `#[mock_function]` attribute macro
/// - The function identifier must be a simple path (e.g., `function_name` or `module::function_name`)
///
/// # Syntax
///
/// ```ignore
/// use_inline_mock!(function_name)(arg1, arg2, ...)
/// ```
///
/// # Expansion
///
/// The macro call:
/// ```ignore
/// use_inline_mock!(divide)(sum(data.clone()), data.len() as f32)
/// ```
/// Expands to:
/// ```ignore
/// {
///     #[cfg(not(test))]
///     { divide }
///     #[cfg(test)]
///     { divide_mock }
/// }(sum(data.clone()), data.len() as f32)
/// ```
///
/// # Note
///
/// You can nest this macro.
///
/// ```ìgnore
/// use_inline_mock!(divide)(
///     use_inline_mock!(sum)(data.clone()),
///     data.len() as f32
/// )
/// ```
#[proc_macro]
pub fn use_inline_mock(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::Expr);

    // Extract the function path
    let fn_path = match &input {
        syn::Expr::Path(path) => path,
        _ => {
            return syn::Error::new_spanned(
                input,
                "use_inline_mock expects a function identifier or path"
            )
            .to_compile_error()
            .into();
        }
    };

    // Get the function name (last segment of the path)
    let fn_name = match fn_path.path.segments.last() {
        Some(segment) => &segment.ident,
        None => {
            return syn::Error::new_spanned(
                &fn_path.path,
                "Could not extract function name from path"
            )
            .to_compile_error()
            .into();
        }
    };

    // Create the mock function name
    let mock_fn_name = syn::Ident::new(&format!("{}_mock", fn_name), fn_name.span());

    // Clone the path for the mock version and replace the last segment
    let mut mock_path = fn_path.clone();
    if let Some(last_segment) = mock_path.path.segments.last_mut() {
        last_segment.ident = mock_fn_name;
    }

    let expanded = quote! {
        {
            #[cfg(not(test))]
            { #fn_path }
            #[cfg(test)]
            { #mock_path }
        }
    };

    TokenStream::from(expanded)
}