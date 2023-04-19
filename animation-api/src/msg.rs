use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsonRpcMessage<T> {
    pub id: Option<usize>,

    #[serde(flatten)]
    pub payload: T,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JsonRpcResult<T, E> {
    Result(T),
    Error(JsonRpcError<E>),
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResponse<T, E> {
    pub id: usize,
    #[serde(flatten)]
    pub result: JsonRpcResult<T, E>,
}

#[repr(i16)]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
pub enum ErrorType {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError<T> {
    pub code: ErrorType,
    pub message: String,
    pub data: Option<T>,
}
