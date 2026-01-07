use quote::quote;
use syn::__private::TokenStream2;
use crate::function_fake::create_fake_implementation::{create_fake_function, create_fake_module};
use crate::function_fake::validate_function::validate_function_fakeable;
use crate::param_utils::create_param_type;
use crate::return_utils::extract_return_type;

mod create_fake_implementation;
mod validate_function;

/// Processes a function and generates the complete fake infrastructure.
///
/// This is the main entry point for the fake_function attribute macro. It takes a function
/// definition and generates:
/// 1. The original function unchanged
/// 2. A fake function with `_fake` suffix (test-only)
/// 3. A fake module with control methods (test-only)
///
/// # Arguments
///
/// * `fake_function` - The function item to create fakes for
///
/// # Returns
///
/// - `Ok(TokenStream2)` - The complete generated code including original and fake infrastructure
/// - `Err(syn::Error)` - If validation fails or the function cannot be faked
///
/// # Validation
///
/// The function validates that:
/// - The function is not async
pub(crate) fn process_fake_function(fake_function: syn::ItemFn) -> syn::Result<TokenStream2> {
    // Validate function is suitable for faking
    validate_function_fakeable(&fake_function)?;

    // Extract function details
    let fn_visibility = fake_function.vis.clone();
    let fn_name = fake_function.sig.ident.clone();
    let fn_inputs = fake_function.sig.inputs.clone();
    let fn_output = fake_function.sig.output.clone();
    let fn_block = fake_function.block.clone();

    // Generate fake function name
    let fake_fn_name = syn::Ident::new(&format!("{}_fake", &fn_name), fn_name.span());

    let params_type = create_param_type(&fn_inputs);
    let return_type = extract_return_type(&fake_function.sig.output);

    let fake_function = create_fake_function(
        fake_fn_name.clone(),
        fn_inputs.clone(),
        fn_output.clone(),
    );
    let fake_module = create_fake_module(
        fake_fn_name,
        params_type,
        return_type
    );

    // Generate the original function, fake function and the fake module
    Ok(quote! {
        #fn_visibility fn #fn_name(#fn_inputs) #fn_output #fn_block

        #[cfg(test)]
        #fake_function

        #[cfg(test)]
        #fake_module
    })
}
