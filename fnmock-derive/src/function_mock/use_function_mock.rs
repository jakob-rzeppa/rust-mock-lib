use quote::quote;
use crate::use_tree_processor::process_use_tree;

/// Processes a use statement and generates conditional imports for mock versions.
///
/// This is the main entry point for the use_function_mock attribute macro. It analyzes
/// a use statement and generates conditional compilation attributes that:
/// - Import the original functions in production builds
/// - Import mock versions (with `_mock` suffix) aliased to original names in test builds
///
/// # Arguments
///
/// * `input` - The use statement to process
///
/// # Returns
///
/// - `Ok(TokenStream2)` - The expanded code with conditional imports
/// - `Err(syn::Error)` - If the use statement cannot be processed
///
/// # Supported Patterns
///
/// - Single import: `use module::function;`
/// - Multiple imports: `use module::{fn1, fn2};`
/// - Nested paths: `use crate::service::fetch_user;`
pub(crate) fn process_use_function_mock(input: syn::ItemUse) -> syn::Result<proc_macro2::TokenStream> {
    // Extract the module path and function name mappings
    let mut base_path = Vec::new();
    let function_mappings = process_use_tree(&input.tree, &mut base_path);

    // Reconstruct the module path as tokens
    let module_path = if base_path.is_empty() {
        quote! {}
    } else {
        quote! { #(#base_path)::* }
    };

    Ok(
        // Generate the appropriate expansion based on number of imports
        if function_mappings.len() == 1 {
            let (fn_name, mock_fn_name) = &function_mappings[0];
            generate_single_import(&input, module_path, fn_name, mock_fn_name)
        } else {
            generate_multiple_imports(&input, module_path, &function_mappings)
        }
    )
}

/// Generates the expanded code for a single function import with mock version.
///
/// Creates conditional compilation attributes that import the original function
/// in production builds and the mock version (aliased to the original name) in test builds.
///
/// # Arguments
///
/// * `input` - The original use statement
/// * `module_path` - The module path tokens (empty if importing from current module)
/// * `fn_name` - The original function name
/// * `mock_fn_name` - The mock function name (with `_mock` suffix)
///
/// # Returns
///
/// Token stream containing:
/// ```ignore
/// #[cfg(not(test))]
/// use original::statement;
/// #[cfg(test)]
/// use module::path::function_mock as function;
/// ```
pub(crate) fn generate_single_import(
    input: &syn::ItemUse,
    module_path: proc_macro2::TokenStream,
    fn_name: &syn::Ident,
    mock_fn_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    quote! {
        #[cfg(not(test))]
        #input
        
        #[cfg(test)]
        use #module_path::#mock_fn_name as #fn_name;
    }
}

/// Generates the expanded code for multiple function imports with mock versions.
///
/// Creates conditional compilation attributes that import the original functions
/// in production builds and the mock versions (aliased to the original names) in test builds.
///
/// # Arguments
///
/// * `input` - The original use statement
/// * `module_path` - The module path tokens (empty if importing from current module)
/// * `function_mappings` - Vector of (original_name, mock_name) tuples
///
/// # Returns
///
/// Token stream containing:
/// ```ignore
/// #[cfg(not(test))]
/// use original::statement;
/// #[cfg(test)]
/// use module::path::{fn1_mock as fn1, fn2_mock as fn2};
/// ```
pub(crate) fn generate_multiple_imports(
    input: &syn::ItemUse,
    module_path: proc_macro2::TokenStream,
    function_mappings: &[(syn::Ident, syn::Ident)],
) -> proc_macro2::TokenStream {
    let mock_alias_mappings: Vec<_> = function_mappings
        .iter()
        .map(|(fn_name, mock_fn_name)| {
            quote! { #mock_fn_name as #fn_name }
        })
        .collect();
    
    quote! {
        #[cfg(not(test))]
        #input
        
        #[cfg(test)]
        use #module_path::{#(#mock_alias_mappings),*};
    }
}
