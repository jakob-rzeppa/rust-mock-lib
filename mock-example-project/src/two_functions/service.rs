use mock_lib::derive::{mock_function};

// Fetches user data from a database
#[mock_function]
pub fn fetch_user(id: u32) -> Result<String, String> {
    Ok(format!("User_{}", id))
}

#[mock_function]
pub fn send_email(user: String, body: String) -> Result<(), String> {
    println!("Send email to {}: {}", user, body);

    Ok(())
}