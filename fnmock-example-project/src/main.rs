mod basic_mock;
mod mock_and_fake;
mod inline_mock;
mod basic_stub;
mod async_fake;
mod async_stub;
mod async_mock;
mod ignore_mock;

fn main() {
    println!("=== fnmock Example Project ===");
    
    // Call example functions to avoid unused warnings
    let _ = basic_mock::db::fetch_user(1);
    basic_mock::handle_user(1);
    
    let _ = mock_and_fake::db::fetch_user(1);
    let _ = mock_and_fake::db::fetch_notes(1);
    let _ = mock_and_fake::handle_user(1);
    
    let _ = inline_mock::fetch_user(1);
    inline_mock::handle_user(1);
    
    let _ = basic_stub::config::get_config();
    let _ = basic_stub::process_config();
    
    // Async functions
    let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let _ = async_fake::db::fetch_user(1).await;
        let _ = async_fake::handle_user(1).await;
        
        let _ = async_stub::config::get_config(1).await;
        let _ = async_stub::process_config(1).await;
        
        let _ = async_mock::db::fetch_user(1).await;
        async_mock::handle_user(1).await;
    });
    
    let _ = ignore_mock::db::save_user(1, "test".to_string(), 0);
    let _ = ignore_mock::db::update_record(1, "test".to_string(), &[1, 2], 0);
    let _ = ignore_mock::db::delete_user(1);
}
