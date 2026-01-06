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

#[cfg(test)]
pub(crate) mod mock {
    use std::cell::RefCell;
    use mock_lib::function_mock::FunctionMock;

    type GenerateStringFromDigitFunction = fn (digit: u8) -> GenerateStringFromDigitResult;
    type GenerateStringIntoDigitParams = u8;
    type GenerateStringFromDigitResult = Result<String, String>;

    thread_local! {
        static GENERATE_STRING_FROM_DIGIT_MOCK: RefCell<FunctionMock<
            GenerateStringFromDigitFunction,
            GenerateStringIntoDigitParams,
            GenerateStringFromDigitResult
        >> = RefCell::new(FunctionMock::new("generate_string_from_digit"));
    }

    pub fn generate_string_from_digit(digit: u8) -> Result<String, String> {
        GENERATE_STRING_FROM_DIGIT_MOCK.with(|mock| {
            let mut mock = mock.borrow_mut();
            mock.call(digit)
        })
    }

    pub(crate) mod generate_string_from_digit {
        pub(crate) fn mock_implementation(new_f: super::GenerateStringFromDigitFunction) {
            super::GENERATE_STRING_FROM_DIGIT_MOCK.with(|mock| { mock.borrow_mut().mock_implementation(new_f) })
        }

        pub(crate) fn clear_mock() {
            super::GENERATE_STRING_FROM_DIGIT_MOCK.with(|mock|{ mock.borrow_mut().clear_mock() })
        }

        pub(crate) fn assert_times(expected_num_of_calls: u32) {
            super::GENERATE_STRING_FROM_DIGIT_MOCK.with(|mock| { mock.borrow().assert_times(expected_num_of_calls) })
        }

        pub(crate) fn assert_with(params: super::GenerateStringIntoDigitParams) {
            super::GENERATE_STRING_FROM_DIGIT_MOCK.with(|mock| { mock.borrow().assert_with(&params) })
        }
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