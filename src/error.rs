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

#[cfg(feature = "reqwest")]
impl ApiError {
    pub async fn from_respponse(resp: reqwest::Response) -> Result<Self, SatoriError> {
        match resp.status() {
            reqwest::StatusCode::BAD_REQUEST => Ok(Self::BadRequest(anyhow::anyhow!(resp
                .text()
                .await
                .map_internal_error()?))),
            reqwest::StatusCode::UNAUTHORIZED => Ok(Self::Unauthorized),
            reqwest::StatusCode::FORBIDDEN => Ok(Self::Forbidden),
            reqwest::StatusCode::NOT_FOUND => Ok(Self::NotFound),
            reqwest::StatusCode::METHOD_NOT_ALLOWED => Ok(Self::MethodNotAllowed),
            s if s.is_server_error() => Ok(Self::ServerError(s.as_u16())),
            _ => Err(SatoriError::InternalError(anyhow::anyhow!(
                "unexpected status code"
            ))),
        }
    }
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

pub trait MapSatoriError<T> {
    fn map_internal_error(self) -> Result<T, SatoriError>;
}

impl<T, E> MapSatoriError<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_internal_error(self) -> Result<T, SatoriError> {
        self.map_err(|e| SatoriError::InternalError(anyhow::Error::new(e)))
    }
}
