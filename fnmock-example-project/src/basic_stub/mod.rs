pub mod config {
    use fnmock::derive::stub_function;

    #[stub_function]
    pub fn get_config() -> String {
        // Real implementation
        "production_config".to_string()
    }
}
use config::get_config;

pub fn process_config() -> String {
    get_config()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::config::{get_config_stub};

    #[test]
    fn test_stub_with_use_stub() {
        // Set up stub
        get_config_stub::setup("test_config".to_string());

        // Call the function that uses the stub
        let result = process_config();

        // Verify result
        assert_eq!(result, "test_config");

        // Clean up
        get_config_stub::clear();
    }
}
