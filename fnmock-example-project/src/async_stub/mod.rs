pub mod config {
    use fnmock::derive::stub_function;

    #[stub_function]
    pub async fn get_config(id: u32) -> String {
        // Real implementation
        format!("production_config: {}", id)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_stub() {
            let res = get_config(2).await;

            assert_eq!(res, "production_config: 2");
        }
    }
}
use config::get_config;

pub async fn process_config(id: u32) -> String {
    get_config(id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::config::{get_config_stub};

    // CAUTION: DO NOT USE MULTIPLE THREADS FOR TESTING (see README.md)
    // #[tokio::test] is single threaded by default
    #[tokio::test]
    async fn test_stub_with_use_stub() {
        // Set up stub
        get_config_stub::setup("test_config".to_string());

        // Call the function that uses the stub
        for i in 0..100 {
            let result = process_config(i).await;

            // Verify result
            assert_eq!(result, "test_config");
        }

        // Clean up
        get_config_stub::clear();
    }
}
