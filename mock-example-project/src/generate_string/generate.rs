use mock_lib::derive::mock_function;

#[mock_function]
pub fn generate_string_from_digit(digit: u8) -> Result<String, String> {
    match digit {
        0 => Ok("Zero".to_string()),
        1 => Ok("One".to_string()),
        2 => Ok("Two".to_string()),
        3 => Ok("Three".to_string()),
        4 => Ok("Four".to_string()),
        5 => Ok("Five".to_string()),
        6 => Ok("Six".to_string()),
        7 => Ok("Seven".to_string()),
        8 => Ok("Eight".to_string()),
        9 => Ok("Nine".to_string()),
        _ => Err("Digit should be between 0 and 9".to_string())
    }
}

// Mocks do not interfere with the tests of the mocked function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one() {
        assert_eq!(generate_string_from_digit(1).unwrap(), String::from("One"));
    }

    #[test]
    fn two() {
        assert_eq!(generate_string_from_digit(2).unwrap(), String::from("Two"));
    }

    // etc.

    #[test]
    fn invalid_number() {
        assert_eq!(generate_string_from_digit(10), Err("Digit should be between 0 and 9".to_string()));
    }
}