use std::fmt::Debug;

pub struct FunctionMock<Params, Result>
where
    Params: Clone + PartialEq + Debug
{
    name: String,
    implementation: Option<fn(Params) -> Result>,
    calls: Vec<Params>
}

impl<Params, Result> FunctionMock<Params, Result>
where
    Params: Clone + PartialEq + Debug,
{
    pub fn new(function_name: &str) -> Self {
        Self {
            name: function_name.to_string(),
            implementation: None,
            calls: Vec::new(),
        }
    }

    // --- Mocking ---

    pub fn get_implementation(&self) -> &Option<fn(Params) -> Result> {
        &self.implementation
    }

    pub fn mock_implementation(&mut self, new_f: fn(Params) -> Result) {
        self.implementation = Some(new_f);
    }

    pub fn clear_mock(&mut self) {
        self.implementation = None;
        self.calls = Vec::new();
    }

    // --- Execute ---

    pub fn call(&mut self, params: Params) -> Result {
        let implementation = self.implementation.as_ref()
            .expect(format!("{} mock not initialized", self.name).as_str());

        self.calls.push(params.clone());
        implementation(params)
    }

    // --- Assert ---

    pub fn assert_times(&self, expected_num_of_calls: u32) {
        assert_eq!(self.calls.len(), expected_num_of_calls as usize,
                   "Expected {} mock to be called {} times, received {}",
                   self.name, self.calls.len(), expected_num_of_calls);
    }

    pub fn assert_with(&self, params: Params) {
        let mut was_called_with = false;

        for called_params in self.calls.iter() {
            if *called_params == params {
                was_called_with = true;
            }
        }

        assert!(was_called_with, "Expected {} mock to be called with {:?}", self.name, params);
    }
}