pub mod db {
    use fnmock::derive::mock_function;

    #[mock_function]
    pub async fn fetch_user(id: u32) -> Result<String, String> {
        // Real implementation
        Ok(format!("user_{}", id))
    }
}

use db::fetch_user;

pub async fn handle_user(id: u32) {
    let _user = fetch_user(id).await;

    // Do something with the user
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::db::fetch_user_mock;

    // CAUTION: DO NOT USE MULTIPLE THREADS FOR TESTING (see README.md)
    // #[tokio::test] is single threaded by default
    #[tokio::test]
    async fn test_with_mock() {
        // Set up mock behavior
        fetch_user_mock::setup(|_| {
            Ok("mock user".to_string())
        });

        handle_user(42).await;

        // Verify behavior
        fetch_user_mock::assert_times(1);
        fetch_user_mock::assert_with(42);

        // No cleanup needed, since mocks are thread / test specific
    }
}