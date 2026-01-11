pub mod db {
    use fnmock::derive::mock_function;

    #[mock_function]
    pub fn fetch_user(id: u32) -> Result<String, String> {
        // Real implementation
        Ok(format!("user_{}", id))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let result = fetch_user(4);

            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result, "user_4".to_string());
        }
    }
}

use db::fetch_user;

pub fn handle_user(id: u32) {
    let _user = fetch_user(id);

    // Do something with the user
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::db::fetch_user_mock;

    #[test]
    fn test_with_mock() {
        // Set up mock behavior
        fetch_user_mock::setup(|_| {
            Ok("mock user".to_string())
        });

        handle_user(42);

        // Verify behavior
        fetch_user_mock::assert_times(1);
        fetch_user_mock::assert_with(42);

        // No cleanup needed, since mocks are thread / test specific
    }
}