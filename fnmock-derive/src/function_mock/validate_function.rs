use crate::param_utils::validate_static_params;

/// Validates that a function is suitable for mocking.
///
/// Performs the following checks:
/// - Function is not async (async functions are not supported)
/// - All parameters are 'static (no references allowed)
///
/// # Arguments
///
/// * `input` - The function item to validate
///
/// # Returns
///
/// - `Ok(())` if the function is valid for mocking
/// - `Err(syn::Error)` with a descriptive error message if validation fails
pub(crate) fn validate_function_mockable(input: &syn::ItemFn) -> syn::Result<()> {
    // Check if function is async and return an error if so
    if input.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            &input.sig.asyncness,
            "mock_function does not support async functions"
        ));
    }

    // Validate that all parameters are 'static (no references)
    validate_static_params(&input.sig.inputs)?;

    Ok(())
}
