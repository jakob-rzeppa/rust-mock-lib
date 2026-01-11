pub mod db {
    use fnmock::derive::fake_function;

    #[fake_function]
    pub async fn fetch_user(id: u32) -> Result<String, String> {
        // Real implementation
        Ok(format!("user_{}", id))
    }

    #[cfg(test)]
    mod mock {
        use super::*;

        #[tokio::test]
        async fn it_works() {
            let res = fetch_user(2).await.unwrap();

            assert_eq!(res, "user_2");
        }
    }
}

use db::fetch_user;

pub async fn handle_user(id: u32) -> Result<String, String> {
    fetch_user(id).await
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::db::fetch_user_fake;

    // CAUTION: DO NOT USE MULTIPLE THREADS FOR TESTING (see README.md)
    // #[tokio::test] is single threaded by default
    #[tokio::test]
    async fn test_with_mock() {
        // Set up mock behavior
        fetch_user_fake::setup(|_| {
            Ok("mock user_42".to_string())
        });

        let res = handle_user(42).await;

        assert_eq!(res.unwrap(), "mock user_42".to_string());
    }
}