use std::{collections::HashMap, fmt};

use animation_api::parameter_schema::ParametersSchema;
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::json;
use url::Url;

#[derive(Clone, PartialEq)]
pub struct Gateway {
    endpoint: Url,
}

#[derive(Debug)]
pub enum GatewayError {
    RequestError { reason: String },
    InvalidResponse { reason: String },
    ApiError { reason: String },
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for GatewayError {}

type Result<T> = std::result::Result<T, GatewayError>;

#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Error { error: String },
    Success(T),
}

fn extract_response<T>(res: ApiResponse<T>) -> Result<T> {
    match res {
        ApiResponse::Success(r) => Ok(r),
        ApiResponse::Error { error } => Err(GatewayError::ApiError { reason: error }),
    }
}

#[cfg(feature = "visualizer")]
#[derive(Deserialize)]
pub struct GetPointsResponse {
    points: Vec<(f32, f32, f32)>,
}

#[derive(Clone, Deserialize)]
pub struct AnimationEntry {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
struct ListAnimationsResponse {
    animations: Vec<AnimationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct Animation {
    pub name: String,
    pub schema: ParametersSchema,
    pub values: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GetParamsResponse {
    animation: Option<Animation>,
}

impl Gateway {
    pub fn new(endpoint: Url) -> Self {
        Self { endpoint }
    }

    fn url(&self, path: &str) -> String {
        self.endpoint.join(path).unwrap().to_string()
    }

    #[cfg(feature = "visualizer")]
    pub fn frames(&self) -> Url {
        let mut endpoint = self.endpoint.clone();
        endpoint.set_scheme("ws").unwrap();
        endpoint.join("frames").unwrap()
    }

    #[cfg(feature = "visualizer")]
    pub async fn get_points(&self) -> Result<Vec<(f32, f32, f32)>> {
        Ok(Request::get(&self.url("points"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<GetPointsResponse>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)?
            .points)
    }

    pub async fn restart_events(&self) -> Result<()> {
        Request::post(&self.url("restart_events"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?;
        Ok(())
    }

    pub async fn list_animations(&self) -> Result<Vec<AnimationEntry>> {
        Ok(Request::get(&self.url("list"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<ListAnimationsResponse>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)?
            .animations)
    }

    pub async fn discover_animations(&self) -> Result<Vec<AnimationEntry>> {
        Ok(Request::post(&self.url("discover"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<ListAnimationsResponse>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)?
            .animations)
    }

    pub async fn switch_animation(&self, animation_id: String) -> Result<Option<Animation>> {
        Ok(Request::post(&self.url("switch"))
            .json(&json!({ "animation": animation_id }))
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<GetParamsResponse>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)?
            .animation)
    }

    pub async fn turn_off(&self) -> Result<()> {
        Request::post(&self.url("turn_off"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?;
        Ok(())
    }

    pub async fn get_params(&self) -> Result<Option<Animation>> {
        Ok(Request::get(&self.url("params"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<GetParamsResponse>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)?
            .animation)
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

    pub async fn reset_params(&self) -> Result<Option<Animation>> {
        Ok(Request::post(&self.url("params/reset"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<GetParamsResponse>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)?
            .animation)
    }

    pub async fn reload_animation(&self) -> Result<Option<Animation>> {
        Ok(Request::post(&self.url("reload"))
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<GetParamsResponse>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)?
            .animation)
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self {
            endpoint: Url::parse("http://localhost/").unwrap(),
        }
    }
}
