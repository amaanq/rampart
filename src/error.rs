//! HTTP error type — maps both internal errors and user-facing
//! validation failures into appropriate responses.

use axum::{
   http::StatusCode,
   response::{
      IntoResponse,
      Response,
   },
};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
   #[error("not found")]
   NotFound,
   #[error("bad request: {0}")]
   BadRequest(String),
   #[error("conflict: {0}")]
   Conflict(String),
   #[error("database error: {0}")]
   Db(#[from] tokio_postgres::Error),
   #[error("pool error: {0}")]
   Pool(#[from] deadpool_postgres::PoolError),
   #[error("template error: {0}")]
   Template(#[from] askama::Error),
   #[error("{0:#}")]
   Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
   fn into_response(self) -> Response {
      let (status, msg) = match &self {
         Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
         Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
         Self::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
         Self::Db(e) => {
            tracing::error!(error = ?e, "db error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "database error".to_string(),
            )
         },
         Self::Pool(e) => {
            tracing::error!(error = ?e, "pool error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "database pool error".to_string(),
            )
         },
         Self::Template(e) => {
            tracing::error!(error = ?e, "template render error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "template error".to_string(),
            )
         },
         Self::Internal(e) => {
            tracing::error!(error = ?e, "internal error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "internal error".to_string(),
            )
         },
      };
      (status, msg).into_response()
   }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
