use quote::quote;
use crate::function_stub::proxy_docs::StubProxyDocs;

/// Generates the original function with stub checking logic injected.
///
/// Creates a function that first checks (in test mode) if a stub implementation has been
/// configured via the stub module. If a stub is set, it calls the stub implementation.
/// Otherwise, it executes the original function body.
///
/// # Arguments
///
/// * `fn_name` - The name of the original function
/// * `fn_visibility` - The visibility modifier of the function (pub, pub(crate), etc.)
/// * `fn_asyncness` - Optional async keyword if the function is async
/// * `fn_inputs` - The function parameters
/// * `fn_output` - The return type
/// * `fn_block` - The original function body to execute when stub is not set
/// * `stub_mod_name` - The name of the stub module containing the stub infrastructure
///
/// # Returns
///
/// Generated token stream for the function with injected stub checking logic
pub(crate) fn create_stub_function(
    fn_name: syn::Ident,
    fn_visibility: syn::Visibility,
    fn_asyncness: Option<syn::token::Async>,
    fn_inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    fn_output: syn::ReturnType,
    fn_block: Box<syn::Block>,
    stub_mod_name: syn::Ident,
) -> proc_macro2::TokenStream {
    let original_fn_stmts = &fn_block.stmts;
    
    quote! {
        #[allow(unused_variables)]
        #fn_visibility #fn_asyncness fn #fn_name(#fn_inputs) #fn_output {
            // Call the stub implementation if set (only in test mode)
            #[cfg(test)]
            if #stub_mod_name::is_set() {
                return #stub_mod_name::get_return_value();
            }

            #(#original_fn_stmts)*
        }
    }
}

/// Generates a stub module containing the stub infrastructure.
///
/// Creates a module with the same name as the stub function that contains:
/// - Thread-local storage for the FunctionStub instance
/// - Proxy functions for stub operations
///
/// # Arguments
///
/// * `stub_fn_name` - The name of the stub module (same as stub function name)
/// * `return_type` - The return type of the function
pub(crate) fn create_stub_module(stub_fn_name: syn::Ident, return_type: syn::Type) -> proc_macro2::TokenStream {
    // Generate documentation using the proxy_docs module
    let docs = StubProxyDocs::new(&stub_fn_name, &return_type);
    let setup_docs = docs.setup_docs();
    let clear_docs = docs.clear_docs();
    let is_set_docs = docs.is_set_docs();
    let get_return_value_docs = docs.get_return_value_docs();
    
    quote! {
        pub(crate) mod #stub_fn_name {
            use super::*;

            thread_local! {
                static STUB: std::cell::RefCell<fnmock::function_stub::FunctionStub<#return_type>> =
                    std::cell::RefCell::new(fnmock::function_stub::FunctionStub::new(stringify!(#stub_fn_name)));
            }

            #setup_docs
            pub(crate) fn setup(return_value: #return_type) {
                STUB.with(|stub| { stub.borrow_mut().setup(return_value) })
            }

            #clear_docs
            pub(crate) fn clear() {
                STUB.with(|stub| { stub.borrow_mut().clear() })
            }

            #is_set_docs
            pub(crate) fn is_set() -> bool {
                STUB.with(|stub| { stub.borrow().is_set() })
            }

            #get_return_value_docs
            pub(crate) fn get_return_value() -> #return_type {
                STUB.with(|stub| { stub.borrow().get_return_value() })
            }
        }
    }
}
