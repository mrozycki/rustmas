use animation_api::parameter_schema::ParametersSchema;
use serde_json::json;

pub trait EventGenerator: Send + Sync {
    fn get_name(&self) -> &str;

    fn restart(&mut self) {}

    fn get_parameter_schema(&self) -> ParametersSchema {
        ParametersSchema {
            parameters: Vec::new(),
        }
    }

    fn set_parameters(&mut self, _parameters: serde_json::Value) -> Result<(), serde_json::Error> {
        Ok(())
    }

    fn get_parameters(&self) -> serde_json::Value {
        json!({})
    }
}
