/// Generates documentation strings for mock proxy functions based on actual function parameters.

use quote::quote;

/// Builds documentation for mock proxy functions.
///
/// Generates parameter documentation, examples, and other descriptive text based on
/// the actual function signature.
pub(crate) struct MockProxyDocs {
    param_docs: Vec<String>,
    ignored_param_docs: Vec<String>,
    setup_example: Vec<String>,
    is_async: bool,
}

impl MockProxyDocs {
    /// Creates documentation for mock proxy functions.
    ///
    /// # Arguments
    ///
    /// * `mock_fn_name` - The name of the mock function/module
    /// * `fn_inputs` - The original function parameters
    /// * `ignore_indices` - Indices of parameters to ignore
    /// * `return_type` - The return type of the function
    /// * `fn_asyncness` - Whether the function is async
    pub(crate) fn new(
        mock_fn_name: &syn::Ident,
        fn_inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        ignore_indices: &[usize],
        return_type: &syn::Type,
        fn_asyncness: Option<syn::token::Async>,
    ) -> Self {
        let all_params: Vec<_> = fn_inputs
            .iter()
            .enumerate()
            .filter_map(|(idx, arg)| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    let name = &pat_type.pat;
                    let ty = &pat_type.ty;
                    let is_ignored = ignore_indices.contains(&idx);
                    Some((name, ty, is_ignored))
                } else {
                    None
                }
            })
            .collect();
        
        // Build parameter documentation
        let param_docs: Vec<String> = all_params
            .iter()
            .filter(|(_, _, is_ignored)| !is_ignored)
            .map(|(name, ty, _)| {
                format!("* `{}: {}` - Parameter value", quote::quote!(#name), quote::quote!(#ty))
            })
            .collect();
        
        let ignored_param_docs: Vec<String> = all_params
            .iter()
            .filter(|(_, _, is_ignored)| *is_ignored)
            .map(|(name, ty, _)| {
                format!("* `{}: {}` - IGNORED (not tracked)", quote::quote!(#name), quote::quote!(#ty))
            })
            .collect();
        
        let setup_example = if all_params.is_empty() {
            vec![
                format!("{}::setup(|| {{", mock_fn_name),
                "    // Custom logic here".to_string(),
                format!("    {}", quote::quote!(#return_type)),
                "});".to_string(),
            ]
        } else {
            let example_params: Vec<_> = all_params
                .iter()
                .filter(|(_, _, is_ignored)| !is_ignored)
                .map(|(name, _, _)| quote::quote!(#name))
                .collect();
            
            let params_pattern = if example_params.len() == 1 {
                quote::quote!(#(#example_params)*)
            } else {
                quote::quote!((#(#example_params),*))
            };
            
            vec![
                format!("{}::setup(|{}| {{", mock_fn_name, quote::quote!(#params_pattern)),
                "    // Custom logic here".to_string(),
                format!("    {}", quote::quote!(#return_type)),
                "});".to_string(),
            ]
        };

        Self {
            param_docs,
            ignored_param_docs,
            setup_example,
            is_async: fn_asyncness.is_some(),
        }
    }

    /// Generates documentation attributes for the `call` function.
    pub(crate) fn call_docs(&self) -> proc_macro2::TokenStream {
        let mut docs = vec![
            quote! { #[doc = "Calls the mock with the provided parameters."] },
            quote! { #[doc = ""] },
            quote! { #[doc = "This function is used internally by the mock function to delegate calls"] },
            quote! { #[doc = "to the mock implementation. If no mock behavior has been set up using `setup()`,"] },
            quote! { #[doc = "this will panic."] },
            quote! { #[doc = ""] },
            quote! { #[doc = "# Parameters"] },
            quote! { #[doc = ""] },
        ];
        
        if self.param_docs.is_empty() {
            docs.push(quote! { #[doc = "No parameters"] });
        } else {
            for param in &self.param_docs {
                docs.push(quote! { #[doc = #param] });
            }
        }
        
        if !self.ignored_param_docs.is_empty() {
            docs.push(quote! { #[doc = ""] });
            docs.push(quote! { #[doc = "# Ignored Parameters"] });
            docs.push(quote! { #[doc = ""] });
            for param in &self.ignored_param_docs {
                docs.push(quote! { #[doc = #param] });
            }
        }
        
        docs.extend(vec![
            quote! { #[doc = ""] },
            quote! { #[doc = "# Returns"] },
            quote! { #[doc = ""] },
            quote! { #[doc = "The return value from the configured mock behavior"] },
            quote! { #[doc = ""] },
            quote! { #[doc = "# Panics"] },
            quote! { #[doc = ""] },
            quote! { #[doc = "Panics if `setup()` has not been called before calling the mock function"] },
        ]);
        
        quote! { #(#docs)* }
    }

    /// Generates documentation attributes for the `setup` function.
    pub(crate) fn setup_docs(&self) -> proc_macro2::TokenStream {
        let mut docs = vec![
            quote! { #[doc = "Sets up the mock behavior."] },
            quote! { #[doc = ""] },
            quote! { #[doc = "Configures the function that will be called when the mock is invoked."] },
            quote! { #[doc = "The provided function receives the parameters (excluding ignored ones) and"] },
            quote! { #[doc = "must return the expected return type."] },
        ];
        
        if self.is_async {
            docs.extend(vec![
                quote! { #[doc = ""] },
                quote! { #[doc = "# Note"] },
                quote! { #[doc = ""] },
                quote! { #[doc = "This function is async, but the mock implementation function must be sync."] },
                quote! { #[doc = "The mock will automatically wrap the return value."] },
            ]);
        }
        
        docs.extend(vec![
            quote! { #[doc = ""] },
            quote! { #[doc = "# Parameters"] },
            quote! { #[doc = ""] },
        ]);
        
        if self.param_docs.is_empty() {
            docs.push(quote! { #[doc = "No parameters"] });
        } else {
            for param in &self.param_docs {
                docs.push(quote! { #[doc = #param] });
            }
        }
        
        if !self.ignored_param_docs.is_empty() {
            docs.push(quote! { #[doc = ""] });
            docs.push(quote! { #[doc = "# Ignored Parameters"] });
            docs.push(quote! { #[doc = ""] });
            for param in &self.ignored_param_docs {
                docs.push(quote! { #[doc = #param] });
            }
        }
        
        docs.extend(vec![
            quote! { #[doc = ""] },
            quote! { #[doc = "# Examples"] },
            quote! { #[doc = ""] },
            quote! { #[doc = "```ignore"] },
        ]);
        
        for line in &self.setup_example {
            docs.push(quote! { #[doc = #line] });
        }
        
        docs.push(quote! { #[doc = "```"] });
        
        quote! { #(#docs)* }
    }

    /// Generates documentation attributes for the `clear` function.
    pub(crate) fn clear_docs(&self) -> proc_macro2::TokenStream {
        quote! {
            #[doc = "Clears the mock state."]
            #[doc = ""]
            #[doc = "Resets the mock by clearing:"]
            #[doc = "- The configured behavior (set via `setup()`)"]
            #[doc = "- The call history"]
            #[doc = ""]
            #[doc = "After calling `clear()`, the mock will panic if invoked before"]
            #[doc = "calling `setup()` again."]
        }
    }

    /// Generates documentation attributes for the `assert_times` function.
    pub(crate) fn assert_times_docs(&self) -> proc_macro2::TokenStream {
        quote! {
            #[doc = "Asserts that the mock was called exactly the expected number of times."]
            #[doc = ""]
            #[doc = "# Parameters"]
            #[doc = ""]
            #[doc = "* `expected_num_of_calls` - The expected number of times the mock should have been called"]
            #[doc = ""]
            #[doc = "# Panics"]
            #[doc = ""]
            #[doc = "Panics if the actual number of calls does not match the expected number"]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = ""]
            #[doc = "```ignore"]
            #[doc = "my_function_mock::assert_times(3); // Expects exactly 3 calls"]
            #[doc = "```"]
        }
    }

    /// Generates documentation attributes for the `assert_with` function.
    pub(crate) fn assert_with_docs(&self) -> proc_macro2::TokenStream {
        let mut docs = vec![
            quote! { #[doc = "Asserts that the mock was called at least once with the specified parameters."] },
            quote! { #[doc = ""] },
            quote! { #[doc = "Checks the call history to verify that at least one call was made with"] },
            quote! { #[doc = "parameters matching the provided values. Only non-ignored parameters need"] },
            quote! { #[doc = "to be provided."] },
            quote! { #[doc = ""] },
            quote! { #[doc = "# Parameters"] },
            quote! { #[doc = ""] },
        ];
        
        if self.param_docs.is_empty() {
            docs.push(quote! { #[doc = "No parameters"] });
        } else {
            for param in &self.param_docs {
                docs.push(quote! { #[doc = #param] });
            }
        }
        
        if !self.ignored_param_docs.is_empty() {
            docs.push(quote! { #[doc = ""] });
            docs.push(quote! { #[doc = "# Ignored Parameters (not required)"] });
            docs.push(quote! { #[doc = ""] });
            docs.push(quote! { #[doc = "The following parameters are ignored and do not need to be provided:"] });
            docs.push(quote! { #[doc = ""] });
            for param in &self.ignored_param_docs {
                docs.push(quote! { #[doc = #param] });
            }
        }
        
        docs.extend(vec![
            quote! { #[doc = ""] },
            quote! { #[doc = "# Panics"] },
            quote! { #[doc = ""] },
            quote! { #[doc = "Panics if no call with matching parameters is found in the call history"] },
            quote! { #[doc = ""] },
        ]);
        
        quote! { #(#docs)* }
    }
}
