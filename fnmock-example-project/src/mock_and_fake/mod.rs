pub mod db {
    use fnmock::derive::{fake_function, mock_function};

    #[fake_function]
    pub fn fetch_user(id: u32) -> Result<String, String> {
        // Real implementation
        Ok(format!("user_{}", id))
    }

    #[mock_function]
    pub fn fetch_notes(id: u32) -> Result<String, String> {
        Ok(format!("notes_{}", id))
    }
}

use db::fetch_user;
use crate::mock_and_fake::db::fetch_notes;

pub fn handle_user(id: u32) -> Result<(), String> {
    fetch_user(id)?;

    fetch_notes(id)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::db::{fetch_notes_mock, fetch_user_fake};

    #[test]
    fn test_handle_invalid_user() {
        // Set up mock behavior
        fetch_user_fake::setup(|_| {
            Err("user not found".to_string())
        });
        fetch_notes_mock::setup(|_| {
            Err("not reached".to_string())
        });

        let err = handle_user(42).unwrap_err();

        assert_eq!(err, "user not found");

        // handle_user returns before calling fetch_notes
        fetch_notes_mock::assert_times(0);

        // No cleanup needed, since fakes are thread / test specific as well
    }
}