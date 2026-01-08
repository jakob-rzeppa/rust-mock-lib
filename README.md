# fnmock

A Rust mocking framework for standalone functions.

[![Crates.io](https://img.shields.io/crates/v/fnmock.svg)](https://crates.io/crates/fnmock)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

fnmock makes testing Rust functions easier by providing three approaches:

-   **Mocks**: Track function calls and enable assertions about call counts and parameters
-   **Fakes**: Provide alternative implementations without call tracking overhead
-   **Stubs**: Return pre-configured values without implementing custom logic

All three approaches support multiple usage patterns to fit your testing needs, from import-level switches to inline call-site selection.

## Features

-   ✅ **Function mocking** - mock and assert function integration in your code
-   ✅ **Function faking** - fake functions for when mocks are unnecessary or can't be used
-   ✅ **Function stubbing** - stub functions with pre-configured return values
-   ✅ **Procedural macros** - mock, fake, and stub functions with little boilerplate
-   ✅ **Zero runtime overhead** - the macros use `#[cfg(test)]` to only compile mocks in test mode
-   ✅ **Thread-isolated** - each test gets its own mock state
-   ⚠️ **Not thread-safe within a test**: If a single test spawns multiple threads that mock the same function, undefined behavior may occur
-   ✅ **No trait requirements** - works with any standalone function

## Installation

Add fnmock to your `Cargo.toml` as a regular dependency:

```toml
[dependencies]
fnmock = ".."
```

**Why not a dev-dependency?** The `#[mock_function]` and `#[fake_function]` macros need to be applied to your production code. However, the macros use conditional compilation (`cfg(test)`) to ensure **zero runtime overhead** in release builds - the mock infrastructure is only compiled in test mode.

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
```

### Basic Stub Example

When you don't need custom logic and just want to return a pre-configured value:

```rust
mod config {
    use fnmock::derive::stub_function;

    #[stub_function]
    pub fn get_config() -> String {
        // Real implementation
        "production_config".to_string()
    }
}

#[use_function_stub]
use config::get_config;

fn process_config() -> String {
    get_config()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::config::get_config_stub;

    #[test]
    fn test_with_stub() {
        // Set up stub to return a specific value
        get_config_stub::setup("test_config".to_string());

        let result = process_config();

        assert_eq!(result, "test_config");

        // Clean up
        get_config_stub::clear();
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
    use super::db::fetch_user_fake;

    #[test]
    fn test_handle_invalid_user() {
        // Set up mock behavior
        fetch_user_fake::setup(|_| {
            Err("user not found".to_string())
        });

        let err = handle_user(42).unwrap_err();

        assert_eq!(err, "user not found");

        // No cleanup needed, since fakes are thread / test specific as well
    }
}
```

## Macros

fnmock supplies three main macro families, one for each approach:

### 1. Attribute Macros (`#[mock_function]` / `#[fake_function]` / `#[stub_function]`)

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
    -   `get_implementation()` - Returns the function pointer of the fake implementation

`#[stub_function]` generates:

-   `send_email_stub()` function
-   `send_email_stub` module with control methods:
    -   `setup(value)` - Set the return value
    -   `clear()` - Reset to default
    -   `get_return_value()` - Returns the configured return value

### 2. Use Statement Macros (`#[use_function_mock]` / `#[use_function_fake]` / `#[use_function_stub]`)

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

The same pattern applies to `#[use_function_fake]` and `#[use_function_stub]`.

### 3. Inline Macros (`use_mock_inline!()` / `use_fake_inline!()` / `use_stub_inline!()`)

Fine-grained control for same-module mocking or specific call sites:

```rust
pub fn calculate_average(data: Vec<f32>) -> f32 {
    use_mock_inline!(sum)(data.clone()) / data.len() as f32
}
```

This expands to use `sum` in production and `sum_mock` in tests at that specific call site.

The same pattern applies to `use_fake_inline!()` and `use_stub_inline!()`.

## Mocks vs Fakes vs Stubs

| Feature              | Mocks                                  | Fakes                      | Stubs                 |
| -------------------- | -------------------------------------- | -------------------------- | --------------------- |
| Call tracking        | ✅ Yes                                 | ❌ No                      | ❌ No                 |
| Assertions           | ✅ Yes (`assert_times`, `assert_with`) | ❌ No                      | ❌ No                 |
| Custom logic         | ✅ Yes (full function)                 | ✅ Yes (full function)     | ❌ No (value only)    |
| Reference parameters | ❌ No (must use owned types)           | ✅ Yes                     | ✅ Yes                |
| Complexity           | Higher                                 | Medium                     | Lower                 |
| Use case             | Verifying behavior                     | Alternative implementation | Pre-configured values |


## Thread Safety

Mocks, fakes, and stubs all use thread-local storage, which means:

✅ **Test isolation**: Each test thread gets its own mock/fake state  
✅ **Parallel tests**: Tests can run in parallel without interference  
⚠️ **Not thread-safe within a test**: If a single test spawns multiple threads that mock the same function, undefined behavior may occur

## Async Functions

fnmock supports async functions! You can apply `#[mock_function]`, `#[fake_function]`, or `#[stub_function]` to async functions just like regular functions.

### Important Constraints

⚠️ **Mock/Fake implementations must be synchronous** - When you set up a mock or fake for an async function, the implementation function you provide must be a regular (non-async) function that returns the appropriate **non-future** type. You cannot use `.await` inside the mock/fake implementations.

⚠️ **Single-threaded testing only** - When testing async functions with mocks/fakes/stubs, you **must** use single-threaded test executors. With tokio, use `#[tokio::test]` (which is single-threaded by default), **not** `#[tokio::test(flavor = "multi_thread")]`.

### Why These Constraints?

1. **Sync implementations**: The underlying storage mechanism requires that the mock/fake function itself be synchronous, since handling async implementations is much more error-prone and not needed for the majority of use cases.

2. **Single-threaded tests**: Because mocks/fakes/stubs use thread-local storage, spawning multiple threads within a single test that access the same mock will lead to undefined behavior. Single-threaded async executors avoid this issue.


## Project Structure

```
fnmock/
├── fnmock/                  # Core library with FunctionMock, FunctionFake, and FunctionStub
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

### For Stubs

-   Functions must be standalone (no `self` parameters)
-   Return type must implement `Clone` (for storing and retrieving the configured value)
-   No parameters required (stubs don't track or use parameters)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Jakob Rzeppa - rzeppa.jakob@gmail.com

## Repository

https://github.com/jakob-rzeppa/rust-helpers-macros
