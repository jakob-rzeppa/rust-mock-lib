/// Generates documentation strings for stub proxy functions based on actual return type.

use quote::quote;

/// Builds documentation for stub proxy functions.
///
/// Generates return type documentation and examples based on the actual return type.
pub(crate) struct StubProxyDocs {
    return_type_str: String,
    setup_example: String,
}

impl StubProxyDocs {
    /// Creates documentation for stub proxy functions.
    ///
    /// # Arguments
    ///
    /// * `stub_fn_name` - The name of the stub function/module
    /// * `return_type` - The return type of the function
    pub(crate) fn new(
        stub_fn_name: &syn::Ident,
        return_type: &syn::Type,
    ) -> Self {
        let return_type_str = quote::quote!(#return_type).to_string();
        let setup_example = format!("{}::setup(/* value of type {} */);", stub_fn_name, return_type_str);
        
        Self {
            return_type_str,
            setup_example,
        }
    }

    /// Generates documentation attributes for the `setup` function.
    pub(crate) fn setup_docs(&self) -> proc_macro2::TokenStream {
        let return_type_str = &self.return_type_str;
        let setup_example = &self.setup_example;
        
        quote! {
            #[doc = "Sets up the stub's return value."]
            #[doc = ""]
            #[doc = "Configures the value that will be returned every time the stub function is called."]
            #[doc = "The value must implement `Clone` since it may be returned multiple times."]
            #[doc = ""]
            #[doc = "# Returns"]
            #[doc = ""]
            #[doc = #return_type_str]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = ""]
            #[doc = "```ignore"]
            #[doc = #setup_example]
            #[doc = "```"]
        }
    }

    /// Generates documentation attributes for the `clear` function.
    pub(crate) fn clear_docs(&self) -> proc_macro2::TokenStream {
        quote! {
            #[doc = "Clears the stub state."]
            #[doc = ""]
            #[doc = "Resets the stub by clearing the configured return value."]
            #[doc = "After calling `clear()`, the stub will panic if invoked before"]
            #[doc = "calling `setup()` again."]
        }
    }

    /// Generates documentation attributes for the `is_set` function.
    pub(crate) fn is_set_docs(&self) -> proc_macro2::TokenStream {
        quote! {
            #[doc = "Checks if the stub has been configured."]
            #[doc = ""]
            #[doc = "Returns `true` if `setup()` has been called and the stub is ready to use,"]
            #[doc = "or `false` if the stub has not been set up or has been cleared."]
            #[doc = ""]
            #[doc = "# Returns"]
            #[doc = ""]
            #[doc = "`bool` - `true` if configured, `false` otherwise"]
        }
    }

    /// Generates documentation attributes for the `get_return_value` function.
    pub(crate) fn get_return_value_docs(&self) -> proc_macro2::TokenStream {
        let return_type_str = &self.return_type_str;
        
        quote! {
            #[doc = "Gets the configured return value."]
            #[doc = ""]
            #[doc = "This function is used internally by the stub function to retrieve"]
            #[doc = "the return value that was configured via `setup()`."]
            #[doc = ""]
            #[doc = "# Returns"]
            #[doc = ""]
            #[doc = #return_type_str]
            #[doc = ""]
            #[doc = "# Panics"]
            #[doc = ""]
            #[doc = "Panics if `setup()` has not been called before calling the stub function"]
        }
    }
}
