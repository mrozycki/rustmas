use animation_api::schema::ParameterSchema;
use rustmas_webapi_client::ParameterValue;
use yew::Properties;

#[derive(Properties, PartialEq, Clone)]
pub struct ParameterControlProps {
    pub schema: ParameterSchema,
    pub value: Option<ParameterValue>,
    pub dummy_update: usize,
}
