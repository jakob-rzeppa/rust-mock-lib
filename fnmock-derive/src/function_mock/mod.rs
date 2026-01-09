use quote::quote;
use syn::__private::TokenStream2;
use crate::function_mock::create_mock_implementation::{create_mock_function, create_mock_module};
use crate::function_mock::validate_function::validate_function_mockable;
use crate::param_utils::{create_param_type, create_tuple_from_param_names, get_param_names};
use crate::return_utils::extract_return_type;

mod create_mock_implementation;
mod validate_function;
mod proxy_docs;
pub(crate) mod mock_args;

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
/// - All parameters are 'static (no references)
/// - Parameters can be cloned, compared, and debugged
pub(crate) fn process_mock_function(mock_function: syn::ItemFn, ignore_params: Vec<String>) -> syn::Result<TokenStream2> {
    // Extract function details
    let fn_visibility = mock_function.vis.clone();
    let fn_asyncness = mock_function.sig.asyncness;
    let fn_name = mock_function.sig.ident.clone();
    let fn_inputs = mock_function.sig.inputs.clone();
    let fn_output = mock_function.sig.output.clone();
    let fn_block = mock_function.block.clone();

    // Generate mock function name
    let mock_fn_name = syn::Ident::new(&format!("{}_mock", &fn_name), fn_name.span());

    // Convert ignore param names to indices
    let ignore_indices = get_ignore_indices(&fn_inputs, &ignore_params)?;

    // Validate function is suitable for mocking (only non-ignored params)
    validate_function_mockable(&mock_function, &ignore_indices)?;

    // Only add the not ignored parameters to the param_types / params_to_tuple
    let params_type = create_param_type(&fn_inputs, &ignore_indices);
    let params_to_tuple = create_tuple_from_param_names(&fn_inputs, &ignore_indices);

    let return_type = extract_return_type(&mock_function.sig.output);

    let filtered_fn_inputs = crate::param_utils::filter_params(&fn_inputs, &ignore_indices);

    let mock_function = create_mock_function(
        mock_fn_name.clone(),
        fn_asyncness.clone(),
        fn_inputs.clone(),
        fn_output.clone(),
        params_to_tuple.clone()
    );
    let mock_module = create_mock_module(
        mock_fn_name,
        params_type,
        return_type,
        &fn_inputs,
        &ignore_indices,
        fn_asyncness.clone(),
        params_to_tuple,
        filtered_fn_inputs
    );

    // Generate the original function, mock function and the mock module
    Ok(quote! {
        #fn_visibility #fn_asyncness fn #fn_name(#fn_inputs) #fn_output #fn_block

        #[cfg(test)]
        #mock_function

        #[cfg(test)]
        #mock_module
    })
}

/// Converts parameter names to their indices.
///
/// Maps each ignored parameter name to its position in the function signature.
fn get_ignore_indices(
    fn_inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    ignore_params: &[String]
) -> syn::Result<Vec<usize>> {
    let param_names = get_param_names(fn_inputs);
    let mut indices = Vec::new();

    for ignore_name in ignore_params {
        let mut found = false;
        for (i, param) in param_names.iter().enumerate() {
            if let syn::Pat::Ident(pat_ident) = param {
                if pat_ident.ident == ignore_name {
                    indices.push(i);
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Parameter '{}' not found in function signature", ignore_name)
            ));
        }
    }

    Ok(indices)
}