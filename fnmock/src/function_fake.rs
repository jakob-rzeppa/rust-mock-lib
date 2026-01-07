/// Struct for faking a function with a custom implementation
///
/// Fakes - in contrast to mocks - do not let you make assertions about if and how the function was called.
///
/// # Generics
///
/// - `Function: 'static + Copy` - the function type
///   - Typically a function pointer like `fn(Args) -> Return`. Closures can be coerced to `fn` types if they do not capture any variables.
///
/// # Usage
///
/// Normally you don't need to interact with the FunctionFake.
/// The usage is automated in the `fnmock-derive::fake_function` macro,
/// and you interact with generated proxy functions.
///
/// The function `calculate` is supposed to be faked for testing.
///
/// ```
/// pub(crate) fn calculate(x: i32, y: i32) -> i32 {
///     x + y
/// }
/// ```
///
/// Now we create a fake function - it should be the same signature,
/// but with `_fake` at the end of the name and the body calls the fake implementation.
///
/// ```
/// pub(crate) fn calculate_fake(x: i32, y: i32) -> i32 {
///     calculate_fake::get_implementation()(x, y)
/// }
/// ```
///
/// Create a module named `calculate_fake` with the fake infrastructure:
///
/// ```
/// pub(crate) mod calculate_fake {
///     use fnmock::function_fake::FunctionFake;
///     
///     type Function = fn(i32, i32) -> i32;
///     
///     thread_local! {
///         static FAKE: std::cell::RefCell<FunctionFake<Function>> =
///             std::cell::RefCell::new(FunctionFake::new("calculate"));
///     }
///
///     // Here we create proxy calls for the fake functions.
///     // This allows us to use `calculate_fake::` for all the important fake functionalities.
///     pub(crate) fn fake_implementation(new_f: Function) {
///         FAKE.with(|fake| { fake.borrow_mut().fake_implementation(new_f) })
///     }
///     
///     pub(crate) fn get_implementation() -> Function {
///         FAKE.with(|fake| { fake.borrow().get_implementation() })
///     }
///
///     // ...
///     // the same for all other fake functions
/// }
/// ```
///
/// # Fields
///
/// - `name` - the name of the function for display purposes when panicking
/// - `implementation` - the fake function implementation or None
pub struct FunctionFake<Function>
where
    Function: 'static + Copy,
{
    name: String,
    implementation: Option<Function>,
}

impl<Function> FunctionFake<Function>
where
    Function: 'static + Copy,
{
    pub fn new(function_name: &str) -> Self {
        Self {
            name: function_name.to_string(),
            implementation: None,
        }
    }

    // --- Faking ---

    pub fn fake_implementation(&mut self, new_f: Function) {
        self.implementation = Some(new_f);
    }

    pub fn clear_fake(&mut self) {
        self.implementation = None;
    }

    pub fn get_implementation(&self) -> Function
    {
        self.implementation.expect(format!("{} fake not initialized", self.name).as_str())
    }
}