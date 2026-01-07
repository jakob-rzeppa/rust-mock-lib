use quote::quote;

/// Generates a mock function that delegates to the mock module's call method.
///
/// Creates a function with the same signature as the original function,
/// but with `_mock` suffix, that calls the mock implementation.
///
/// # Arguments
///
/// * `mock_fn_name` - The name of the mock function (original name with `_mock` suffix)
/// * `fn_inputs` - The function parameters
/// * `fn_output` - The return type
/// * `params_to_tuple` - Token stream that converts parameters into a tuple for the mock
pub(crate) fn create_mock_function(
    mock_fn_name: syn::Ident,
    fn_inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    fn_output: syn::ReturnType,
    params_to_tuple: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        pub(crate) fn #mock_fn_name(#fn_inputs) #fn_output {
            #mock_fn_name::call(#params_to_tuple)
        }
    }
}

/// Generates a mock module containing the mock infrastructure.
///
/// Creates a module with the same name as the mock function that contains:
/// - Type aliases for parameters and return type
/// - Thread-local storage for the FunctionMock instance
/// - Proxy functions for all mock operations
///
/// # Arguments
///
/// * `mock_fn_name` - The name of the mock module (same as mock function name)
/// * `params_type` - The type representing the function parameters (single type or tuple)
/// * `return_type` - The return type of the function
pub(crate) fn create_mock_module(mock_fn_name: syn::Ident, params_type: syn::Type, return_type: syn::Type) -> proc_macro2::TokenStream {
    quote! {
        pub(crate) mod #mock_fn_name {
            type Params = #params_type;
            type Return = #return_type;
            const FUNCTION_NAME: &str = stringify!(#mock_fn_name);

            thread_local! {
                static MOCK: std::cell::RefCell<fnmock::function_mock::FunctionMock<
                    Params,
                    Return,
                >> = std::cell::RefCell::new(fnmock::function_mock::FunctionMock::new(FUNCTION_NAME));
            }

            pub(crate) fn call(params: Params) -> Return {
                MOCK.with(|mock| {
                    mock.borrow_mut().call(params)
                })
            }

            pub(crate) fn mock_implementation(new_f: fn(Params) -> Return) {
                MOCK.with(|mock| {
                    mock.borrow_mut().mock_implementation(new_f)
                })
            }

            pub(crate) fn clear_mock() {
                MOCK.with(|mock|{
                    mock.borrow_mut().clear_mock()
                })
            }

            pub(crate) fn assert_times(expected_num_of_calls: u32) {
                MOCK.with(|mock| {
                    mock.borrow().assert_times(expected_num_of_calls)
                })
            }

            pub(crate) fn assert_with(params: Params) {
                MOCK.with(|mock| {
                    mock.borrow().assert_with(params)
                })
            }
        }
    }
}