use crate::param_utils::validate_static_params;

/// Validates that a function is suitable for mocking.
///
/// Performs the following checks:
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
    // Validate that all parameters are 'static (no references)
    validate_static_params(&input.sig.inputs)?;

    Ok(())
}
