# fnmock

A Rust mocking framework for standalone functions.

[![Crates.io](https://img.shields.io/crates/v/fnmock.svg)](https://crates.io/crates/fnmock)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

fnmock makes testing Rust functions easier by providing two approaches:

-   **Mocks**: Track function calls and enable assertions about call counts and parameters
-   **Fakes**: Provide alternative implementations without call tracking overhead

Both approaches support multiple usage patterns to fit your testing needs, from import-level switches to inline call-site mocking.

## Features

-   ✅ **Function mocking** - mock and assert function integration in your code
-   ✅ **Function faking** - fake functions for when mocks are unnecessary or can't be used
-   ✅ **Procedural macros** - mock functions with little boilerplate
-   ✅ **Thread-isolated** - each test gets its own mock state
-   ⚠️ **Not thread-safe within a test**: If a single test spawns multiple threads that mock the same function, undefined behavior may occur
-   ✅ **No trait requirements** - works with any standalone function

### Basic Mock Example

```rust
mod db {
    use fnmock::derive::mock_function;
    
    #[mock_function]
    pub fn fetch_user(id: u32) -> Result<String, String> {
        // Real implementation
        Ok(format!("user_{}", id))
    }
}

#[use_function_mock]
use db::fetch_user;

fn handle_user(id: u32) {
    let user = fetch_user(id);
    
    // Do something with the user
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::db::fetch_user_mock;

    #[test]
    fn test_with_mock() {
        // Set up mock behavior
        fetch_user_mock::mock_implementation(|_| {
            Ok("mock user")
        });

        let result = handle_user(42);

        // Verify behavior
        fetch_user_mock::assert_times(1);
        fetch_user_mock::assert_with(42);
        
        // No cleanup needed, since mocks are thread / test specific
    }
}
```

### Basic Fake Example

Fnmock mocks can't handle non-static parameters, since they can't always be stored safely. 
That's where fakes come into play.

```rust
mod db {
    use fnmock::derive::fake_function;

    #[fake_function]
    pub fn fetch_user(id: u32) -> Result<String, String> {
        // Real implementation
        Ok(format!("user_{}", id))
    }
}

#[use_function_fake]
use db::fetch_user;

fn handle_user(id: u32) -> Result<(), String> {
    let user = fetch_user(id)?;

    // Do something with the user
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::db::fetch_user_mock;

    #[test]
    fn test_handle_invalid_user() {
        // Set up mock behavior
        fetch_user_fake::fake_implementation(|_| {
            Err("user not found")
        });

        let err = handle_user(42).unwrap_err;
        
        assert_eq!(err, "user not found");

        // No cleanup needed, since fakes are thread / test specific as well
    }
}
```

## Macros

fnmock supplies three main macros:

### 1. Attribute Macros (`#[mock_function]` / `#[fake_function]`)

The foundational approach that generates mock/fake functions:

```rust
#[mock_function]
pub fn send_email(to: String, body: String) -> Result<(), String> {
    // Real implementation
    Ok(())
}
```

`#[mock_function]` generates:

-   `send_email_mock()` function
-   `send_email_mock` module with control methods:
    -   `mock_implementation(fn)` - Set custom behavior
    -   `clear_mock()` - Reset to default
    -   `call(params)` - Calls the mock implementation (params must be supplied as a tuple)
    -   `assert_times(n)` - Verify call count
    -   `assert_with(params)` - Verify parameters

`#[fake_function]` generates:

-   `send_email_fake()` function
-   `send_email_fake` module with control methods:
    -   `fake_implementation(fn)` - Set custom behavior
    -   `clear_fake()` - Reset to default
    -   `get_implementation()` - Returns the function pointer of the mock implementation

### 2. Use Statement Macros (`#[use_function_mock]` / `#[use_function_fake]`)

Automatically switch between real and mock/fake versions based on build mode:

```rust
// In module A
#[mock_function]
pub fn fetch_data(id: u32) -> String {
    format!("data_{}", id)
}

// In module B
#[use_function_mock]
use crate::module_a::fetch_data;

pub fn process_data(id: u32) -> String {
    fetch_data(id).to_uppercase() // Uses real in production, mock in tests
}
```

In production builds: imports `fetch_data`  
In test builds: imports `fetch_data_mock as fetch_data`

### 3. Inline Macros (`use_mock_inline!()` / `use_fake_inline!()`)

Fine-grained control for same-module mocking or specific call sites:

```rust
pub fn calculate_average(data: Vec<f32>) -> f32 {
    use_mock_inline!(sum)(data.clone()) / data.len() as f32
}
```

This expands to use `sum` in production and `sum_mock` in tests at that specific call site.

## Mocks vs Fakes

| Feature              | Mocks                                 | Fakes              |
|----------------------|---------------------------------------|--------------------|
| Call tracking        | ✅ Yes                                 | ❌ No               |
| Assertions           | ✅ Yes (`assert_times`, `assert_with`) | ❌ No               |
| Reference parameters | ❌ No (must use owned types)           | ✅ Yes              |
| Complexity           | Higher                                | Lower              |
| Use case             | Verifying behavior                    | Simple replacement |

## Project Structure

```
fnmock/
├── fnmock/                  # Core library with FunctionMock and FunctionFake
│   ├── src/
│   └── Cargo.toml
├── fnmock-derive/           # Procedural macros
│   ├── src/
│   └── Cargo.toml
├── fnmock-example-project/  # Examples
│   ├── src/
│   ├── README.md
│   └── Cargo.toml
└── Cargo.toml               # Workspace configuration
```

## Requirements

### For Mocks

-   Function parameters must implement:
    -   `Clone` - for storing call history
    -   `Debug` - for assertion error messages
    -   `PartialEq` - for parameter assertions
    -   `'static` - no borrowed references (use owned types like `String`)
-   Functions must be standalone (no `self` parameters)

### For Fakes

-   Functions must be standalone (no `self` parameters)
-   No trait requirements on parameters (**references allowed!**)

## Thread Safety

Both mocks and fakes use thread-local storage, which means:

✅ **Test isolation**: Each test thread gets its own mock/fake state  
✅ **Parallel tests**: Tests can run in parallel without interference  
⚠️ **Not thread-safe within a test**: If a single test spawns multiple threads that mock the same function, undefined behavior may occur

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Jakob Rzeppa - rzeppa.jakob@gmail.com

## Repository

https://github.com/jakob-rzeppa/rust-helpers-macros
