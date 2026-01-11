pub mod db {
    use fnmock::derive::mock_function;

    // Mock with ignore parameter
    #[mock_function(ignore = [timestamp])]
    pub fn save_user(id: u32, name: String, timestamp: i64) -> Result<(), String> {
        println!("Saving user {} with name {} at {}", id, name, timestamp);
        Ok(())
    }

    // Another example with multiple ignored params
    #[mock_function(ignore = [updated_at, created_at])]
    pub fn update_record(id: u32, value: String, created_at: &[u32], updated_at: i64) -> Result<(), String> {
        println!("Updating record {} with value {} (created: {:?}, updated: {})", 
                 id, value, created_at, updated_at);
        Ok(())
    }

    // Mock without ignore for comparison
    #[mock_function]
    pub fn delete_user(id: u32) -> Result<(), String> {
        println!("Deleting user {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::db::{save_user, save_user_mock, update_record, update_record_mock, delete_user, delete_user_mock};

    #[test]
    fn test_save_user_with_ignored_timestamp() {
        // Set up mock behavior
        save_user_mock::setup(|params| {
            println!("Mock called with: {:?}", params);
            Ok(())
        });

        // Call with different timestamps
        let _ = save_user(1, "Alice".to_string(), 1000);
        let _ = save_user(1, "Alice".to_string(), 2000);

        // Assert was called 2 times
        save_user_mock::assert_times(2);

        save_user_mock::assert_with(1, "Alice".to_string());

        save_user_mock::assert_with(1, "Alice".to_string());
    }

    #[test]
    fn test_update_record_with_ignored_timestamps() {
        // Set up mock behavior
        update_record_mock::setup(|_| Ok(()));

        // Call with different timestamps
        let _ = update_record(42, "test".to_string(), &[1, 2, 3], 2000);

        update_record_mock::assert_times(1);

        // Check that id and value match, ignoring the timestamp fields
        update_record_mock::assert_with(42, "test".to_string());
    }

    #[test]
    #[should_panic]
    fn test_assert_with_ignore_fails_on_non_ignored_params() {
        update_record_mock::setup(|_| Ok(()));
        
        let _ = update_record(42, "test".to_string(), &[1, 2, 3], 2000);

        // This should fail because id doesn't match (42 != 99)
        update_record_mock::assert_with(99, "test".to_string());
    }

    #[test]
    fn test_mock_without_ignore_uses_regular_assert_with() {
        // Mocks without ignore parameter work normally
        delete_user_mock::setup(|_| Ok(()));

        let _ = delete_user(123);

        delete_user_mock::assert_times(1);
        delete_user_mock::assert_with(123);
    }

    #[test]
    fn test_multiple_calls_with_different_ignored_values() {
        save_user_mock::setup(|_| Ok(()));

        // Make multiple calls with same id and name, but different timestamps
        let _ = save_user(5, "Bob".to_string(), 100);
        let _ = save_user(5, "Bob".to_string(), 200);
        let _ = save_user(5, "Bob".to_string(), 300);

        save_user_mock::assert_times(3);

        // All three calls should match when checking with any timestamp (ignored)
        save_user_mock::assert_with(5, "Bob".to_string());
    }
}
