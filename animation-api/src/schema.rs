use std::collections::HashMap;

use lightfx::Color;
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
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ParameterSchema>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub custom_triggers: Vec<EnumOption>,
}

pub trait GetSchema {
    fn schema() -> Vec<ParameterSchema>;
}

impl GetSchema for () {
    fn schema() -> Vec<ParameterSchema> {
        Vec::new()
    }
}

pub trait GetEnumOptions {
    fn enum_options() -> Vec<EnumOption>;
}

impl GetEnumOptions for () {
    fn enum_options() -> Vec<EnumOption> {
        Vec::new()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
pub enum ParameterValue {
    Number(f64),
    Color(Color),
    EnumOption(String),
}

#[cfg(feature = "yew")]
impl yew::html::ImplicitClone for ParameterValue {}

impl ParameterValue {
    pub fn number(&self) -> Option<f64> {
        if let Self::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn color(&self) -> Option<&Color> {
        if let Self::Color(c) = self {
            Some(c)
        } else {
            None
        }
    }

    pub fn enum_option(&self) -> Option<&str> {
        if let Self::EnumOption(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Configuration {
    pub id: String,
    pub name: String,
    pub schema: ConfigurationSchema,
    pub values: HashMap<String, ParameterValue>,
}
