use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, Ident};

/// Processes a function path expression and generates the conditional selection code.
///
/// Takes a function path and creates a block that conditionally evaluates to either
/// the original function or the modified version (with custom suffix) based on the test configuration.
///
/// # Arguments
///
/// * `input` - The expression passed to the macro (should be a path expression)
/// * `suffix` - The suffix to append to the function name (e.g., "_mock" or "_fake")
/// * `macro_name` - The name of the macro for error messages (e.g., "use_mock_inline" or "use_fake_inline")
///
/// # Returns
///
/// - `Ok(TokenStream2)` - The generated conditional code block
/// - `Err(syn::Error)` - If the input is not a valid function path
///
/// # Generated Code
///
/// ```ignore
/// {
///     #[cfg(not(test))]
///     { original::path::function }
///     #[cfg(test)]
///     { original::path::function_suffix }
/// }
/// ```
pub(crate) fn process_inline(
    input: Expr,
    suffix: &str,
    macro_name: &str,
) -> syn::Result<TokenStream2> {
    // Extract the function path
    let fn_path = match input {
        Expr::Path(path) => path,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                format!("{} expects a function identifier or path", macro_name)
            ));
        }
    };

    // Get the function name (last segment of the path)
    let fn_name = match fn_path.path.segments.last() {
        Some(segment) => &segment.ident,
        None => {
            return Err(syn::Error::new_spanned(
                &fn_path.path,
                "Could not extract function name from path"
            ));
        }
    };

    // Create the modified function name with suffix
    let modified_fn_name = Ident::new(&format!("{}{}", fn_name, suffix), fn_name.span());

    // Clone the path for the modified version and replace the last segment
    let mut modified_path = fn_path.clone();
    if let Some(last_segment) = modified_path.path.segments.last_mut() {
        last_segment.ident = modified_fn_name;
    }

    Ok(quote! {
        {
            #[cfg(not(test))]
            { #fn_path }
            #[cfg(test)]
            { #modified_path }
        }
    })
}
