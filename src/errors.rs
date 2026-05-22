use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid json")]
    InvalidJson,

    #[error("Unsupported Auth type")]
    UnsupportedAuthType,

    #[error("A request was rate limited")]
    RateLimited,

    #[error("Missing token in auth response")]
    MissingTokenInResponse,    
    
    #[error("Missing user_id in auth response")]
    MissingUserIdInResponse,    
    
    #[error("Unrecognized status code from `auth_metadata`")]
    AuthMetadataQueryUnrecognizedCode,
}

#[derive(Debug, Error)]
pub enum MatrixError {
    #[error("Matrix Error")]
    MatrixError(String, Option<String>), // Error code, error message
}

impl MatrixError {
    pub fn json(json: Value) -> Self {
        MatrixError::MatrixError(json.get("errcode").and_then(|f| f.as_str()).unwrap_or("Not an Error").to_string(), json.get("error").and_then(|f| f.as_str()).and_then(|t| Some(t.to_string())))
    }
}
