use std::collections::HashMap;

use gloo_net::http::Request;
use lightfx::schema::ParametersSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone, PartialEq)]
pub struct Gateway {
    endpoint: String,
}

pub enum GatewayError {
    RequestError { reason: String },
    InvalidRequest,
    InvalidResponse,
}

type Result<T> = std::result::Result<T, GatewayError>;

#[derive(Clone, Deserialize)]
pub struct Animation {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Default)]
struct ListAnimationsResponse {
    animations: Vec<Animation>,
}
#[derive(Deserialize, Default, Debug)]
pub struct GetParamsResponse {
    pub schema: ParametersSchema,
    pub values: HashMap<String, serde_json::Value>,
}

impl Gateway {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_owned(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.endpoint, path)
    }

    pub async fn list_animations(&self) -> Result<Vec<Animation>> {
        Ok(Request::get(&self.url("list"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ListAnimationsResponse>()
            .await
            .map_err(|_| GatewayError::InvalidResponse)?
            .animations)
    }

    pub async fn switch_animation(&self, animation_name: String) -> Result<()> {
        let _ = Request::post(&self.url("switch"))
            .json(&json!({ "animation": animation_name }))
            .map_err(|_| GatewayError::InvalidRequest)?
            .send()
            .await;
        Ok(())
    }

    pub async fn get_params(&self) -> Result<GetParamsResponse> {
        Ok(Request::get(&self.url("params"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<GetParamsResponse>()
            .await
            .map_err(|_| GatewayError::InvalidResponse)?)
    }

    pub async fn set_params(&self, params: &serde_json::Value) -> Result<()> {
        let _ = Request::post(&self.url("params"))
            .json(params)
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .send()
            .await;
        Ok(())
    }

    pub async fn save_params(&self) -> Result<()> {
        let _ = Request::post(&self.url("params/save")).send().await;
        Ok(())
    }

    pub async fn reset_params(&self) -> Result<GetParamsResponse> {
        Ok(Request::post(&self.url("params/reset"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<GetParamsResponse>()
            .await
            .map_err(|_| GatewayError::InvalidResponse)?)
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost/".to_owned(),
        }
    }
}
