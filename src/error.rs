use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("bad request")]
    BadRequest(#[from] anyhow::Error),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("server error ({0})")]
    ServerError(u16),
}

#[derive(Debug, Error)]
pub enum SatoriError {
    #[error(transparent)]
    ApiError(#[from] ApiError),
    #[error("invalid bot")]
    InvalidBot,
    #[error("internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}
