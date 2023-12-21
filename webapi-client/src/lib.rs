use std::{collections::HashMap, fmt};

use animation_api::parameter_schema::ParametersSchema;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use url::Url;

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

#[derive(Clone)]
pub struct RustmasApiClient {
    endpoint: Url,
    client: reqwest::Client,
}

impl RustmasApiClient {
    pub fn new(endpoint: Url) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        self.endpoint.join(path).unwrap().to_string()
    }

    async fn send_request<T: DeserializeOwned>(request: reqwest::RequestBuilder) -> Result<T> {
        request
            .send()
            .await
            .map_err(|e| GatewayError::RequestError {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<T>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(extract_response)
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, json: &serde_json::Value) -> Result<T> {
        Self::send_request::<T>(self.client.post(&self.url(path)).json(json)).await
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        Self::send_request::<T>(self.client.get(&self.url(path))).await
    }

    #[cfg(feature = "visualizer")]
    pub fn frames(&self) -> Url {
        let mut endpoint = self.endpoint.clone();
        endpoint.set_scheme("ws").unwrap();
        endpoint.join("frames").unwrap()
    }

    #[cfg(feature = "visualizer")]
    pub async fn get_points(&self) -> Result<Vec<(f32, f32, f32)>> {
        Ok(self.get::<GetPointsResponse>("points").await?.points)
    }

    pub async fn restart_events(&self) -> Result<()> {
        self.post::<()>("restart_events", &json!(())).await?;
        Ok(())
    }

    pub async fn list_animations(&self) -> Result<Vec<AnimationEntry>> {
        Ok(self.get::<ListAnimationsResponse>("list").await?.animations)
    }

    pub async fn discover_animations(&self) -> Result<Vec<AnimationEntry>> {
        Ok(self
            .post::<ListAnimationsResponse>("discover", &json!(()))
            .await?
            .animations)
    }

    pub async fn switch_animation(&self, animation_id: String) -> Result<Option<Animation>> {
        Ok(self
            .post::<GetParamsResponse>("switch", &json!({ "animation": animation_id}))
            .await?
            .animation)
    }

    pub async fn turn_off(&self) -> Result<()> {
        self.post::<()>("turn_off", &json!(())).await
    }

    pub async fn get_params(&self) -> Result<Option<Animation>> {
        Ok(self.get::<GetParamsResponse>("params").await?.animation)
    }

    pub async fn set_params(&self, params: &serde_json::Value) -> Result<()> {
        self.post::<()>("params", params).await
    }

    pub async fn save_params(&self) -> Result<()> {
        let _ = self.post::<()>("params/save", &json!(())).await;
        Ok(())
    }

    pub async fn reset_params(&self) -> Result<Option<Animation>> {
        Ok(self
            .post::<GetParamsResponse>("params/reset", &json!(()))
            .await?
            .animation)
    }

    pub async fn reload_animation(&self) -> Result<Option<Animation>> {
        Ok(self
            .post::<GetParamsResponse>("reload", &json!(()))
            .await?
            .animation)
    }
}

impl Default for RustmasApiClient {
    fn default() -> Self {
        Self::new(Url::parse("http://localhost/").unwrap())
    }
}

impl PartialEq for RustmasApiClient {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint == other.endpoint
    }
}
