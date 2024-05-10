use std::collections::HashMap;

use serde::{de::DeserializeOwned, Serialize};
use url::Url;

pub use webapi_model::{
    Animation, Configuration, GetEventGeneratorSchemaResponse, GetParametersResponse,
    GetPointsResponse, ListAnimationsResponse, ParameterValue, SwitchAnimationRequest,
};
use webapi_model::{
    ApiResponse, SetAnimationParametersRequest, SetEventGeneratorParametersRequest,
    SwitchAnimationResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("invalid request: {reason}")]
    InvalidRequest { reason: String },

    #[error("API returned an invalid response: {reason}")]
    InvalidResponse { reason: String },

    #[error("API returned an error: {reason}")]
    ApiError { reason: String },
}

type Result<T> = std::result::Result<T, GatewayError>;

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
            .map_err(|e| GatewayError::InvalidRequest {
                reason: e.to_string(),
            })?
            .json::<ApiResponse<T>>()
            .await
            .map_err(|e| GatewayError::InvalidResponse {
                reason: e.to_string(),
            })
            .and_then(|res| {
                let res: std::result::Result<T, String> = res.into();
                res.map_err(|reason| GatewayError::ApiError { reason })
            })
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, req: &impl Serialize) -> Result<T> {
        Self::send_request::<T>(self.client.post(&self.url(path)).json(req)).await
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
        self.post("events/restart", &()).await
    }

    pub async fn events_schema(&self) -> Result<Vec<Configuration>> {
        Ok(self
            .get::<GetEventGeneratorSchemaResponse>("events/schema")
            .await?
            .event_generators)
    }

    pub async fn set_events_params(
        &self,
        params: &HashMap<String, HashMap<String, ParameterValue>>,
    ) -> Result<()> {
        self.post(
            "events/values",
            &SetEventGeneratorParametersRequest {
                event_generators: params.clone(),
            },
        )
        .await
    }

    pub async fn list_animations(&self) -> Result<Vec<Animation>> {
        Ok(self.get::<ListAnimationsResponse>("list").await?.animations)
    }

    pub async fn discover_animations(&self) -> Result<Vec<Animation>> {
        Ok(self
            .post::<ListAnimationsResponse>("discover", &())
            .await?
            .animations)
    }

    pub async fn switch_animation(&self, animation_id: String) -> Result<Configuration> {
        Ok(self
            .post::<SwitchAnimationResponse>(
                "switch",
                &SwitchAnimationRequest {
                    animation: animation_id,
                    params: None,
                },
            )
            .await?
            .animation)
    }

    pub async fn turn_off(&self) -> Result<()> {
        self.post("turn_off", &()).await
    }

    pub async fn get_params(&self) -> Result<Option<Configuration>> {
        Ok(self.get::<GetParametersResponse>("params").await?.animation)
    }

    pub async fn set_params(&self, params: &HashMap<String, ParameterValue>) -> Result<()> {
        self.post(
            "params",
            &SetAnimationParametersRequest {
                values: params.clone(),
            },
        )
        .await
    }

    pub async fn save_params(&self) -> Result<()> {
        let _ = self.post::<()>("params/save", &()).await;
        Ok(())
    }

    pub async fn reset_params(&self) -> Result<Configuration> {
        Ok(self
            .post::<SwitchAnimationResponse>("params/reset", &())
            .await?
            .animation)
    }

    pub async fn reload_animation(&self) -> Result<Configuration> {
        Ok(self
            .post::<SwitchAnimationResponse>("reload", &())
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
