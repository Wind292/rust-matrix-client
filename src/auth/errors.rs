use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid json")]
    InvalidJson,

    #[error("Unsupported Auth type")]
    UnsupportedAuthType
}