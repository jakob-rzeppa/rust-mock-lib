//! Processing logic for **use statement syntax trees**.
//!
//! This module handles the transformation of use statements to extract function names
//! and generate corresponding mock function names.

use syn;

/// Recursively processes a use tree to extract function names and generate mock names.
///
/// This function traverses the syntax tree of a use statement, collecting the module path in the `base_path` vector
/// and extracting function names. For each function, it generates a corresponding mock
/// function name by appending `_mock`.
///
/// # Arguments
///
/// * `tree` - The use tree node to process
/// * `base_path` - Accumulator for the module path segments (e.g., ["crate", "module"])
///
/// # Returns
///
/// A vector of tuples where each tuple contains:
/// * Original function identifier (e.g., `fetch_user`)
/// * Generated mock function identifier (e.g., `fetch_user_mock`)
///
/// # Examples
///
/// For `use module::function;`:
/// - Returns: `[(function, function_mock)]`
/// - base_path after: `["module"]`
///
/// For `use module::{fn1, fn2};`:
/// - Returns: `[(fn1, fn1_mock), (fn2, fn2_mock)]`
/// - base_path after: `["module"]`
///
/// # Panics
///
/// Panics if the use tree contains unsupported patterns like glob imports (`*`)
/// or renamed imports (`as`).
pub(crate) fn process_use_tree(
    tree: &syn::UseTree,
    base_path: &mut Vec<syn::Ident>,
) -> Vec<(syn::Ident, syn::Ident)> {
    match tree {
        // Handle path segments: module::submodule::...
        syn::UseTree::Path(path) => {
            base_path.push(path.ident.clone());
            process_use_tree(&path.tree, base_path)
        }
        // Handle individual function name
        syn::UseTree::Name(name) => {
            let fn_name = name.ident.clone();
            let mock_fn_name = syn::Ident::new(
                &format!("{}_mock", fn_name),
                fn_name.span()
            );
            vec![(fn_name, mock_fn_name)]
        }
        // Handle grouped imports: {fn1, fn2, fn3}
        syn::UseTree::Group(group) => {
            let mut function_mappings = Vec::new();
            for item in &group.items {
                // Clone base_path for each item to handle nested groups correctly
                let mut item_path = base_path.clone();
                function_mappings.extend(process_use_tree(item, &mut item_path));
            }
            function_mappings
        }
        // Glob imports and renamed imports are not supported
        _ => panic!(
            "use_function_mock only supports simple path and grouped imports. \
             Glob imports (*) and renamed imports (as) are not supported."
        ),
    }
}
