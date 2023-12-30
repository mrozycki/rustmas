use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct EnumOption {
    pub name: String,
    pub description: Option<String>,
    pub value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValueSchema {
    Number { min: f64, max: f64, step: f64 },
    Color,
    Enum { values: Vec<EnumOption> },
    Speed,
    Percentage,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ParameterSchema {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(flatten)]
    pub value: ValueSchema,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct ConfigurationSchema {
    pub parameters: Vec<ParameterSchema>,
}

pub trait GetSchema {
    fn schema() -> ConfigurationSchema;
}

impl GetSchema for () {
    fn schema() -> ConfigurationSchema {
        ConfigurationSchema {
            parameters: Vec::new(),
        }
    }
}

pub trait GetEnumOptions {
    fn enum_options() -> Vec<EnumOption>;
}
