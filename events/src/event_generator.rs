use std::collections::HashMap;

use animation_api::schema::{ParameterSchema, ParameterValue};

pub trait EventGenerator: Send + Sync {
    fn get_name(&self) -> &str;

    fn restart(&mut self) {}

    fn get_schema(&self) -> Vec<ParameterSchema> {
        Vec::new()
    }

    fn set_parameters(
        &mut self,
        _parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(), serde_json::Error> {
        Ok(())
    }

    fn get_parameters(&self) -> HashMap<String, ParameterValue> {
        HashMap::new()
    }
}
