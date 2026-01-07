use quote::quote;
use syn::__private::TokenStream2;
use crate::function_mock::create_mock_implementation::{create_mock_function, create_mock_module};
use crate::function_mock::validate_function::validate_function_mockable;
use crate::param_utils::{create_param_type, create_tuple_from_param_names, validate_static_params};
use crate::return_utils::extract_return_type;

pub(crate) mod use_function_mock;
pub(crate) mod use_mock_inline;
mod create_mock_implementation;
mod validate_function;

/// Processes a function and generates the complete mock infrastructure.
///
/// This is the main entry point for the mock_function attribute macro. It takes a function
/// definition and generates:
/// 1. The original function unchanged
/// 2. A mock function with `_mock` suffix (test-only)
/// 3. A mock module with control and assertion methods (test-only)
///
/// # Arguments
///
/// * `mock_function` - The function item to create mocks for
///
/// # Returns
///
/// - `Ok(TokenStream2)` - The complete generated code including original and mock infrastructure
/// - `Err(syn::Error)` - If validation fails or the function cannot be mocked
///
/// # Validation
///
/// The function validates that:
/// - The function is not async
/// - All parameters are 'static (no references)
/// - Parameters can be cloned, compared, and debugged
pub(crate) fn process_mock_function(mock_function: syn::ItemFn) -> syn::Result<TokenStream2> {
    // Validate function is suitable for mocking
    validate_function_mockable(&mock_function)?;

    // Extract function details
    let fn_visibility = mock_function.vis.clone();
    let fn_name = mock_function.sig.ident.clone();
    let fn_inputs = mock_function.sig.inputs.clone();
    let fn_output = mock_function.sig.output.clone();
    let fn_block = mock_function.block.clone();

    // Generate mock function name
    let mock_fn_name = syn::Ident::new(&format!("{}_mock", &fn_name), fn_name.span());

    let params_type = create_param_type(&fn_inputs);
    let params_to_tuple = create_tuple_from_param_names(&fn_inputs);
    let return_type = extract_return_type(&mock_function.sig.output);

    let mock_function = create_mock_function(
        mock_fn_name.clone(),
        fn_inputs.clone(),
        fn_output.clone(),
        params_to_tuple
    );
    let mock_module = create_mock_module(
        mock_fn_name,
        params_type,
        return_type
    );

    // Generate the original function, mock function and the mock module
    Ok(quote! {
        #fn_visibility fn #fn_name(#fn_inputs) #fn_output #fn_block

        #[cfg(test)]
        #mock_function

        #[cfg(test)]
        #mock_module
    })
}