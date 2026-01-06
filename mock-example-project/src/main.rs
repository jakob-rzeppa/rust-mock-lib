#[cfg(not(test))]
use crate::generate_string::generate_string_from_digit;
#[cfg(test)]
use crate::generate_string::mock::generate_string_from_digit;

mod generate_string;

fn generate_output(digit: u8) -> Result<String, String> {
    Ok(format!("Output: {}", generate_string_from_digit(digit)?))
}

fn main() {
    println!("{}", generate_output(2).unwrap());
}

#[cfg(test)]
mod tests {
    use crate::generate_output;
    use crate::generate_string::mock::generate_string_from_digit;

    #[test]
    fn it_works() {
        generate_string_from_digit::mock_implementation(|_: u8| {
            Ok("Mock Output".to_string())
        });

        let res = generate_output(2);

        generate_string_from_digit::assert_times(1);
        generate_string_from_digit::assert_with(2);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "Output: Mock Output");
    }

    #[test]
    fn it_propagates_the_error() {
        generate_string_from_digit::mock_implementation(|_: u8| {
            Err("Mock Error".to_string())
        });

        let res = generate_output(2);

        generate_string_from_digit::assert_times(1);
        generate_string_from_digit::assert_with(2);
        assert_eq!(res, Err("Mock Error".to_string()));
    }
}