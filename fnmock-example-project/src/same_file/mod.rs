use fnmock::derive::{mock_function, use_mock_inline};

#[mock_function]
fn sum(nums: Vec<f32>) -> f32 {
    let mut sum = 0.0;
    for num in nums {
        sum += num;
    }
    sum
}

#[mock_function]
fn divide(numerator: f32, denominator: f32) -> Result<f32, String> {
    if numerator == 0.0 {
        Err(String::from("numerator must be 0.0"))
    } else {
        Ok(numerator / denominator)
    }
}

fn find_average(data: Vec<f32>) -> Result<f32, String> {
    use_mock_inline!(divide)(
        use_mock_inline!(sum)(data.clone()),
        data.len() as f32
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn find_average_calls_sum_and_divide_with_the_right_params() {
        divide_mock::mock_implementation(|_| {
            Ok(5.2f32)
        });
        sum_mock::mock_implementation(|_| {
            200f32
        });

        let data = vec![0.1, 0.2, 0.3, 0.4];

        let result = find_average(data);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(5.2f32, result);

        divide_mock::assert_times(1);
        divide_mock::assert_with((200f32, 4f32));

        sum_mock::assert_times(1);
        sum_mock::assert_with(vec![0.1, 0.2, 0.3, 0.4]);
    }
}