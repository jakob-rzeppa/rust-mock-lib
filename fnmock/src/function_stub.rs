/// Struct for stubbing a function with predetermined return values
///
/// Stubs - in contrast to mocks and fakes - provide canned responses without behavior verification or custom logic.
/// They simply return predetermined values to allow tests to proceed.
///
/// # Generics
///
/// - `ReturnType: 'static + Clone` - the return type of the stubbed function
///   - Must be cloneable since the stub may be called multiple times with the same return value
///
/// # Usage
///
/// Normally you don't need to interact with the FunctionStub directly.
/// The usage is automated in the `fnmock-derive::stub_function` macro,
/// and you interact with generated proxy functions.
///
/// The function `get_config` is supposed to be stubbed for testing.
///
/// ```
/// pub(crate) fn get_config() -> String {
///     // Production code that reads from file or environment
///     std::fs::read_to_string("config.json").unwrap()
/// }
/// ```
///
/// Now we create a stub function - it should be the same signature,
/// but with `_stub` at the end of the name and the body calls the stub return value.
///
/// ```
/// pub(crate) fn get_config_stub() -> String {
///     get_config_stub::get_return_value()
/// }
/// ```
///
/// Create a module named `get_config_stub` with the stub infrastructure:
///
/// ```
/// pub(crate) mod get_config_stub {
///     use fnmock::function_stub::FunctionStub;
///     
///     thread_local! {
///         static STUB: std::cell::RefCell<FunctionStub<String>> =
///             std::cell::RefCell::new(FunctionStub::new("get_config"));
///     }
///
///     // Here we create proxy calls for the stub functions.
///     // This allows us to use `get_config_stub::` for all the important stub functionalities.
///     pub(crate) fn setup(return_value: String) {
///         STUB.with(|stub| { stub.borrow_mut().setup(return_value) })
///     }
///     
///     pub(crate) fn get_return_value() -> String {
///         STUB.with(|stub| { stub.borrow().get_return_value() })
///     }
///
///     pub(crate) fn clear() {
///         STUB.with(|stub| { stub.borrow_mut().clear() })
///     }
/// }
/// ```
///
/// # Fields
///
/// - `name` - the name of the function for display purposes when panicking
/// - `return_value` - the stubbed return value or None
pub struct FunctionStub<ReturnType>
where
    ReturnType: 'static + Clone,
{
    name: String,
    return_value: Option<ReturnType>,
}

impl<ReturnType> FunctionStub<ReturnType>
where
    ReturnType: 'static + Clone,
{
    pub fn new(function_name: &str) -> Self {
        Self {
            name: function_name.to_string(),
            return_value: None,
        }
    }

    // --- Stubbing ---

    pub fn setup(&mut self, new_r: ReturnType) {
        self.return_value = Some(new_r.clone());
    }

    pub fn clear(&mut self) {
        self.return_value = None;
    }

    pub fn is_set(&self) -> bool {
        self.return_value.is_some()
    }

    pub fn get_return_value(&self) -> ReturnType {
        self.return_value.clone().expect(format!("{} stub not initialized", self.name).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_stub_with_correct_name() {
        let stub: FunctionStub<i32> = FunctionStub::new("test_function");
        assert_eq!(stub.name, "test_function");
        assert!(stub.return_value.is_none());
    }

    #[test]
    fn test_setup_sets_return_value() {
        let mut stub: FunctionStub<i32> = FunctionStub::new("get_value");
        stub.setup(42);
        assert!(stub.return_value.is_some());
    }

    #[test]
    fn test_get_return_value_returns_configured_value() {
        let mut stub: FunctionStub<i32> = FunctionStub::new("get_value");
        stub.setup(42);
        
        let result = stub.get_return_value();
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "get_value stub not initialized")]
    fn test_get_return_value_panics_when_not_initialized() {
        let stub: FunctionStub<i32> = FunctionStub::new("get_value");
        stub.get_return_value();
    }

    #[test]
    fn test_clear_resets_return_value() {
        let mut stub: FunctionStub<i32> = FunctionStub::new("get_value");
        stub.setup(42);
        
        assert!(stub.return_value.is_some());
        
        stub.clear();
        
        assert!(stub.return_value.is_none());
    }

    #[test]
    fn test_stub_can_be_updated() {
        let mut stub: FunctionStub<i32> = FunctionStub::new("get_value");
        stub.setup(42);
        
        let result1 = stub.get_return_value();
        assert_eq!(result1, 42);
        
        stub.setup(100);
        let result2 = stub.get_return_value();
        assert_eq!(result2, 100);
    }

    #[test]
    fn test_with_string_return_type() {
        let mut stub: FunctionStub<String> = FunctionStub::new("get_config");
        stub.setup("test_config".to_string());
        
        let result = stub.get_return_value();
        assert_eq!(result, "test_config");
    }

    #[test]
    fn test_with_vec_return_type() {
        let mut stub: FunctionStub<Vec<i32>> = FunctionStub::new("get_numbers");
        stub.setup(vec![1, 2, 3, 4, 5]);
        
        let result = stub.get_return_value();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_with_option_return_type() {
        let mut stub: FunctionStub<Option<i32>> = FunctionStub::new("get_optional");
        stub.setup(Some(42));
        
        let result = stub.get_return_value();
        assert_eq!(result, Some(42));
        
        stub.setup(None);
        let result2 = stub.get_return_value();
        assert_eq!(result2, None);
    }

    #[test]
    fn test_with_result_return_type() {
        let mut stub: FunctionStub<Result<i32, String>> = FunctionStub::new("get_result");
        stub.setup(Ok(42));
        
        let result = stub.get_return_value();
        assert_eq!(result, Ok(42));
        
        stub.setup(Err("error occurred".to_string()));
        let result2 = stub.get_return_value();
        assert_eq!(result2, Err("error occurred".to_string()));
    }

    #[test]
    fn test_multiple_get_return_value_calls() {
        let mut stub: FunctionStub<i32> = FunctionStub::new("get_value");
        stub.setup(42);
        
        let result1 = stub.get_return_value();
        let result2 = stub.get_return_value();
        let result3 = stub.get_return_value();
        
        assert_eq!(result1, 42);
        assert_eq!(result2, 42);
        assert_eq!(result3, 42);
    }

    #[test]
    fn test_with_tuple_return_type() {
        let mut stub: FunctionStub<(i32, String)> = FunctionStub::new("get_pair");
        stub.setup((42, "answer".to_string()));
        
        let result = stub.get_return_value();
        assert_eq!(result, (42, "answer".to_string()));
    }

    #[test]
    fn test_with_custom_struct() {
        #[derive(Clone, Debug, PartialEq)]
        struct Config {
            port: u16,
            host: String,
        }
        
        let mut stub: FunctionStub<Config> = FunctionStub::new("get_config");
        stub.setup(Config {
            port: 8080,
            host: "localhost".to_string(),
        });
        
        let result = stub.get_return_value();
        assert_eq!(result.port, 8080);
        assert_eq!(result.host, "localhost");
    }

    #[test]
    fn test_function_name_preserved() {
        let stub: FunctionStub<i32> = FunctionStub::new("my_custom_function");
        assert_eq!(stub.name, "my_custom_function");
    }
}