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
///     pub(crate) fn setup(new_f: Function) {
///         FAKE.with(|fake| { fake.borrow_mut().setup(new_f) })
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

    pub fn setup(&mut self, new_f: Function) {
        self.implementation = Some(new_f);
    }

    pub fn clear(&mut self) {
        self.implementation = None;
    }

    pub fn is_set(&self) -> bool {
        self.implementation.is_some()
    }

    pub fn get_implementation(&self) -> Function
    {
        self.implementation.expect(format!("{} fake not initialized", self.name).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper fake functions for testing
    fn add_fake_implementation(a: i32, b: i32) -> i32 {
        a + b
    }

    fn multiply_fake_implementation(a: i32, b: i32) -> i32 {
        a * b
    }

    fn string_concat_fake_implementation(a: String, b: String) -> String {
        format!("{}{}", a, b)
    }

    fn sum_fake_implementation(name: &[u32]) -> u32 {
        name.iter().sum()
    }

    #[test]
    fn test_new_creates_fake_with_correct_name() {
        let fake: FunctionFake<fn(i32, i32) -> i32> = FunctionFake::new("test_function");
        assert_eq!(fake.name, "test_function");
        assert!(fake.implementation.is_none());
    }

    #[test]
    fn test_fake_implementation_sets_function() {
        let mut fake: FunctionFake<fn(i32, i32) -> i32> = FunctionFake::new("add");
        fake.setup(add_fake_implementation);
        assert!(fake.implementation.is_some());
    }

    #[test]
    fn test_get_implementation_returns_function() {
        let mut fake: FunctionFake<fn(i32, i32) -> i32> = FunctionFake::new("add");
        fake.setup(add_fake_implementation);
        
        let implementation = fake.get_implementation();
        let result = implementation(5, 3);
        assert_eq!(result, 8);
    }

    #[test]
    #[should_panic(expected = "add fake not initialized")]
    fn test_get_implementation_panics_when_not_initialized() {
        let fake: FunctionFake<fn(i32, i32) -> i32> = FunctionFake::new("add");
        fake.get_implementation();
    }

    #[test]
    fn test_clear_fake_resets_implementation() {
        let mut fake: FunctionFake<fn(i32, i32) -> i32> = FunctionFake::new("add");
        fake.setup(add_fake_implementation);
        
        assert!(fake.implementation.is_some());
        
        fake.clear();
        
        assert!(fake.implementation.is_none());
    }

    #[test]
    fn test_fake_can_be_replaced() {
        let mut fake: FunctionFake<fn(i32, i32) -> i32> = FunctionFake::new("math");
        fake.setup(add_fake_implementation);
        
        let implementation1 = fake.get_implementation();
        let result1 = implementation1(5, 3);
        assert_eq!(result1, 8);
        
        fake.setup(multiply_fake_implementation);
        let implementation2 = fake.get_implementation();
        let result2 = implementation2(5, 3);
        assert_eq!(result2, 15);
    }

    #[test]
    fn test_with_string_parameters() {
        let mut fake: FunctionFake<fn(String, String) -> String> = FunctionFake::new("concat");
        fake.setup(string_concat_fake_implementation);
        
        let implementation = fake.get_implementation();
        let result = implementation("Hello".to_string(), "World".to_string());
        assert_eq!(result, "HelloWorld");
    }

    #[test]
    fn test_with_reference_parameter() {
        let mut fake: FunctionFake<fn(&[u32]) -> u32> = FunctionFake::new("sum");
        fake.setup(sum_fake_implementation);

        let vec = vec![1, 2, 3];
        
        let implementation = fake.get_implementation();
        let result = implementation(vec.as_slice());
        assert_eq!(result, 6);
    }

    #[test]
    fn test_with_unit_return_type() {
        fn void_fake(_x: i32) -> () {
            // Do nothing
        }
        
        let mut fake: FunctionFake<fn(i32) -> ()> = FunctionFake::new("void_fn");
        fake.setup(void_fake);
        
        let implementation = fake.get_implementation();
        implementation(42); // Should not panic
    }

    #[test]
    fn test_with_result_return_type() {
        fn divide_fake(a: i32, b: i32) -> Result<i32, String> {
            if b == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(a / b)
            }
        }
        
        let mut fake: FunctionFake<fn(i32, i32) -> Result<i32, String>> = FunctionFake::new("divide");
        fake.setup(divide_fake);
        
        let implementation = fake.get_implementation();
        
        let result1 = implementation(10, 2);
        assert_eq!(result1, Ok(5));
        
        let result2 = implementation(10, 0);
        assert_eq!(result2, Err("Division by zero".to_string()));
    }

    #[test]
    fn test_with_option_return_type() {
        fn safe_divide_fake(a: i32, b: i32) -> Option<i32> {
            if b == 0 {
                None
            } else {
                Some(a / b)
            }
        }
        
        let mut fake: FunctionFake<fn(i32, i32) -> Option<i32>> = FunctionFake::new("safe_divide");
        fake.setup(safe_divide_fake);
        
        let implementation = fake.get_implementation();
        
        let result1 = implementation(10, 2);
        assert_eq!(result1, Some(5));
        
        let result2 = implementation(10, 0);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_multiple_get_implementation_calls() {
        let mut fake: FunctionFake<fn(i32, i32) -> i32> = FunctionFake::new("add");
        fake.setup(add_fake_implementation);
        
        let impl1 = fake.get_implementation();
        let impl2 = fake.get_implementation();
        
        assert_eq!(impl1(5, 3), 8);
        assert_eq!(impl2(10, 20), 30);
    }

    #[test]
    fn test_function_name_preserved() {
        let fake: FunctionFake<fn(i32) -> i32> = FunctionFake::new("my_custom_function");
        assert_eq!(fake.name, "my_custom_function");
    }
}