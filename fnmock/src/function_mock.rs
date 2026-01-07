use std::fmt::Debug;

/// Struct containing the Data for mocking a Function
///
/// The functions parameters can't contain non 'static variables.
///
/// # Generics
///
/// - `Params: Clone + PartialEq + Debug + 'static` - the parameters of the mocked function as a tuple
/// - `Result` - the result of the function
///
/// # Usage
///
/// Normally you don't need to interact with the FunctionMock.
/// The usage is automated in the `fnmock-derive::mock_function` macro,
/// and you interact with generated proxy functions.
///
/// The function send_email is supposed to be mocked.
///
/// ```
/// pub(crate) fn send_email(user: String, body: String) -> Result<(), String> {
///     print!("Send email to {0}: {1}\n", user, body);
///     Ok(())
/// }
/// ```
///
/// Now we create a mock function of send_email - it should be the same function,
/// but with _mock at the end of the name and the body replaced with `send_email_mock::call`.
/// It is important, when passing the parameters, to put them in a tuple or the function will break.
///
/// ```
/// pub(crate) fn send_email_mock(user: String, body: String) -> Result<(), String> {
///     send_email_mock::call((user, body))
/// }
/// ```
///
/// But where does `send_email_mock::call` come from? Now we create a module named `send_email_mock`.
///
/// ```
/// pub(crate) mod send_email_mock {
///     type Params = (String, String); // The params of the function in a tuple
///     type Return = Result<(), String>; // The return type
///     const FUNCTION_NAME: &str = "send_email"; // the name of the function
///
///     // Tests in rust are run parallel in different threads. thread_local! lets us create a
///     // static mock implementation for each test, so they don't interfere with each other.
///     thread_local! {
///         // the mock is not accessible from the outside
///         static MOCK: RefCell<FunctionMock<
///             Params,
///             Return,
///         >> = RefCell::new(FunctionMock::new(FUNCTION_NAME));
///     }
///
///     // Here we create proxy calls for the mock functions.
///     // This allows us to use `send_email_mock::` for all the important mock functionalities.
///     pub(crate) fn call(params: Params) -> Return {
///         MOCK.with(|mock| { mock.borrow_mut().call(params) })
///     }
///     pub(crate) fn mock_implementation(new_f: fn(Params) -> Return) {
///         MOCK.with(|mock| { mock.borrow_mut().mock_implementation(new_f) })
///     }
///     // ...
///     // the same for all other mock functions
/// }
/// ```
///
/// # Fields
///
/// - `name` - the name of the function for display purposes when asserting
/// - `implementation` - the mock function with the params in a tuple or None
/// - `calls` - vector to hold all calls to the mock
pub struct FunctionMock<Params, Result>
where
    Params: Clone + PartialEq + Debug + 'static
{
    name: String,
    implementation: Option<fn(Params) -> Result>,
    calls: Vec<Params>
}

impl<Params, Result> FunctionMock<Params, Result>
where
    Params: Clone + PartialEq + Debug + 'static,
{
    pub fn new(function_name: &str) -> Self {
        Self {
            name: function_name.to_string(),
            implementation: None,
            calls: Vec::new(),
        }
    }

    // --- Mocking ---

    pub fn mock_implementation(&mut self, new_f: fn(Params) -> Result) {
        self.implementation = Some(new_f);
    }

    pub fn clear_mock(&mut self) {
        self.implementation = None;
        self.calls = Vec::new();
    }

    // --- Execute ---

    pub fn call(&mut self, params: Params) -> Result {
        let implementation = self.implementation.as_ref()
            .expect(format!("{} mock not initialized", self.name).as_str());

        self.calls.push(params.clone());
        implementation(params)
    }

    // --- Assert ---

    pub fn assert_times(&self, expected_num_of_calls: u32) {
        assert_eq!(self.calls.len(), expected_num_of_calls as usize,
                   "Expected {} mock to be called {} times, received {}",
                   self.name, self.calls.len(), expected_num_of_calls);
    }

    pub fn assert_with(&self, params: Params) {
        let mut was_called_with = false;

        for called_params in self.calls.iter() {
            if *called_params == params {
                was_called_with = true;
            }
        }

        assert!(was_called_with, "Expected {} mock to be called with {:?}", self.name, params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper mock function for testing
    fn add_mock_implementation(params: (i32, i32)) -> i32 {
        params.0 + params.1
    }

    fn multiply_mock_implementation(params: (i32, i32)) -> i32 {
        params.0 * params.1
    }

    fn string_concat_mock_implementation(params: (String, String)) -> String {
        format!("{}{}", params.0, params.1)
    }

    #[test]
    fn test_new_creates_mock_with_correct_name() {
        let mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("test_function");
        assert_eq!(mock.name, "test_function");
        assert!(mock.implementation.is_none());
        assert!(mock.calls.is_empty());
    }

    #[test]
    fn test_mock_implementation_sets_function() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        assert!(mock.implementation.is_some());
    }

    #[test]
    fn test_call_executes_mocked_function() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        
        let result = mock.call((5, 3));
        assert_eq!(result, 8);
    }

    #[test]
    #[should_panic(expected = "add mock not initialized")]
    fn test_call_panics_when_not_initialized() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.call((5, 3));
    }

    #[test]
    fn test_call_records_parameters() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        
        mock.call((5, 3));
        mock.call((10, 20));
        
        assert_eq!(mock.calls.len(), 2);
        assert_eq!(mock.calls[0], (5, 3));
        assert_eq!(mock.calls[1], (10, 20));
    }

    #[test]
    fn test_clear_mock_resets_state() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        mock.call((5, 3));
        mock.call((10, 20));

        assert_eq!(mock.calls[0], (5, 3));
        assert_eq!(mock.calls[1], (10, 20));
        
        mock.clear_mock();
        
        assert!(mock.implementation.is_none());
        assert!(mock.calls.is_empty());
    }

    #[test]
    fn test_mock_can_be_replaced() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("math");
        mock.mock_implementation(add_mock_implementation);
        
        let result1 = mock.call((5, 3));
        assert_eq!(result1, 8);
        
        mock.mock_implementation(multiply_mock_implementation);
        let result2 = mock.call((5, 3));
        assert_eq!(result2, 15);
    }

    #[test]
    fn test_assert_times_passes_with_correct_count() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        
        mock.call((1, 2));
        mock.call((3, 4));
        mock.call((5, 6));
        
        mock.assert_times(3);
    }

    #[test]
    #[should_panic(expected = "Expected add mock to be called 2 times, received 5")]
    fn test_assert_times_fails_with_wrong_count() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        
        mock.call((1, 2));
        mock.call((3, 4));
        
        mock.assert_times(5);
    }

    #[test]
    fn test_assert_times_with_zero_calls() {
        let mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.assert_times(0);
    }

    #[test]
    fn test_assert_with_passes_when_called_with_params() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        
        mock.call((5, 3));
        mock.call((10, 20));
        
        mock.assert_with((5, 3));
        mock.assert_with((10, 20));
    }

    #[test]
    #[should_panic(expected = "Expected add mock to be called with (7, 8)")]
    fn test_assert_with_fails_when_not_called_with_params() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        
        mock.call((5, 3));
        mock.assert_with((7, 8));
    }

    #[test]
    fn test_assert_with_finds_params_among_multiple_calls() {
        let mut mock: FunctionMock<(i32, i32), i32> = FunctionMock::new("add");
        mock.mock_implementation(add_mock_implementation);
        
        mock.call((1, 1));
        mock.call((2, 2));
        mock.call((3, 3));
        mock.call((4, 4));
        
        mock.assert_with((3, 3));
    }

    #[test]
    fn test_with_string_parameters() {
        let mut mock: FunctionMock<(String, String), String> = FunctionMock::new("concat");
        mock.mock_implementation(string_concat_mock_implementation);
        
        let result = mock.call(("Hello".to_string(), "World".to_string()));
        assert_eq!(result, "HelloWorld");
        
        mock.assert_times(1);
        mock.assert_with(("Hello".to_string(), "World".to_string()));
    }

    #[test]
    fn test_with_single_parameter() {
        fn double_mock(params: i32) -> i32 {
            params * 2
        }
        
        let mut mock: FunctionMock<i32, i32> = FunctionMock::new("double");
        mock.mock_implementation(double_mock);
        
        let result = mock.call(5);
        assert_eq!(result, 10);
        
        mock.assert_times(1);
        mock.assert_with(5);
    }

    #[test]
    fn test_with_unit_return_type() {
        fn void_mock(_params: i32) -> () {
            // Do nothing
        }
        
        let mut mock: FunctionMock<i32, ()> = FunctionMock::new("void_fn");
        mock.mock_implementation(void_mock);
        
        mock.call(42);
        mock.assert_times(1);
        mock.assert_with(42);
    }

    #[test]
    fn test_with_result_return_type() {
        fn result_mock(params: (i32, i32)) -> Result<i32, String> {
            if params.1 == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(params.0 / params.1)
            }
        }
        
        let mut mock: FunctionMock<(i32, i32), Result<i32, String>> = FunctionMock::new("divide");
        mock.mock_implementation(result_mock);
        
        let result1 = mock.call((10, 2));
        assert_eq!(result1, Ok(5));
        
        let result2 = mock.call((10, 0));
        assert_eq!(result2, Err("Division by zero".to_string()));
        
        mock.assert_times(2);
    }

    #[test]
    fn test_multiple_calls_preserve_order() {
        let mut mock: FunctionMock<i32, i32> = FunctionMock::new("identity");
        mock.mock_implementation(|x| x);
        
        mock.call(1);
        mock.call(2);
        mock.call(3);
        
        assert_eq!(mock.calls, vec![1, 2, 3]);
    }
}