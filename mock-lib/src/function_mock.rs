use std::fmt::Debug;

/// Struct containing the Data for mocking a Function
///
/// # Generics
///
/// - `Params: Clone + PartialEq + Debug` - the parameters of the mocked function as a tuple
/// - `Result` - the result of the function
///
/// # Usage
///
/// Normally you don't need to interact with the FunctionMock.
/// The usage is automated in the `mock-lib-derive::mock_function` macro.
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
    Params: Clone + PartialEq + Debug
{
    name: String,
    implementation: Option<fn(Params) -> Result>,
    calls: Vec<Params>
}

impl<Params, Result> FunctionMock<Params, Result>
where
    Params: Clone + PartialEq + Debug,
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