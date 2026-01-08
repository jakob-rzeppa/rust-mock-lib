mod config {
    use fnmock::derive::stub_function;

    #[stub_function]
    pub(crate) async fn get_config() -> String {
        // Real implementation
        "production_config".to_string()
    }
}

use fnmock::derive::use_function_stub;

#[use_function_stub]
use config::get_config;

async fn process_config() -> String {
    get_config().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::config::{get_config_stub};

    // CAUTION: DO NOT USE MULTIPLE THREADS FOR TESTING (see README.md)
    // #[tokio::test] is single threaded by default
    #[tokio::test]
    async fn test_stub_with_use_function_stub() {
        // Set up stub
        get_config_stub::setup("test_config".to_string());

        // Call the function that uses the stub
        for _ in 0..100 {
            let result = process_config().await;

            // Verify result
            assert_eq!(result, "test_config");
        }

        // Clean up
        get_config_stub::clear();
    }
}
