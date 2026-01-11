use fnmock::derive::{mock_function};

#[mock_function]
pub fn fetch_user(id: u32) -> Result<String, String> {
    // Real implementation
    Ok(format!("user_{}", id))
}

pub fn handle_user(id: u32) {
    // Since fetch_user is in the same module as handle_user, we don't need to import it.
    // That's why we can't use #[use_mock] and have to use the mock inline
    let _user = fetch_user(id);

    // Do something with the user
}


#[cfg(test)]
mod tests {
    use super::*;

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