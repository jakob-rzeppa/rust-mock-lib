use quote::quote;
use syn::__private::TokenStream2;
use crate::function_stub::create_stub_implementation::{create_stub_function, create_stub_module};
use crate::return_utils::extract_return_type;

mod create_stub_implementation;

/// Processes a function and generates the complete stub infrastructure.
///
/// This is the main entry point for the stub_function attribute macro. It takes a function
/// definition and generates:
/// 1. The original function unchanged
/// 2. A stub function with `_stub` suffix (test-only)
/// 3. A stub module with control methods (test-only)
///
/// # Arguments
///
/// * `stub_function` - The function item to create stubs for
///
/// # Returns
///
/// - `Ok(TokenStream2)` - The complete generated code including original and stub infrastructure
/// - `Err(syn::Error)` - If validation fails or the function cannot be stubbed
pub(crate) fn process_stub_function(stub_function: syn::ItemFn) -> syn::Result<TokenStream2> {
    // Extract function details
    let fn_visibility = stub_function.vis.clone();
    let fn_asyncness = stub_function.sig.asyncness;
    let fn_name = stub_function.sig.ident.clone();
    let fn_inputs = stub_function.sig.inputs.clone();
    let fn_output = stub_function.sig.output.clone();
    let fn_block = stub_function.block.clone();

    // Generate stub function name
    let stub_fn_name = syn::Ident::new(&format!("{}_stub", &fn_name), fn_name.span());

    let return_type = extract_return_type(&stub_function.sig.output);

    let stub_function = create_stub_function(
        stub_fn_name.clone(),
        fn_asyncness.clone(),
        fn_inputs.clone(),
        fn_output.clone(),
    );
    let stub_module = create_stub_module(
        stub_fn_name,
        return_type
    );

    // Generate the original function, stub function and the stub module
    Ok(quote! {
        #fn_visibility #fn_asyncness fn #fn_name(#fn_inputs) #fn_output #fn_block

        #[cfg(test)]
        #stub_function

        #[cfg(test)]
        #stub_module
    })
}
