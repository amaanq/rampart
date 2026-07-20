//! HTTP error type — maps both internal errors and user-facing
//! validation failures into appropriate responses.

use axum::{
   Json,
   http::{
      StatusCode,
      header,
   },
   response::{
      IntoResponse,
      Response,
   },
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
   #[error("not found")]
   NotFound,
   #[error("bad request: {0}")]
   BadRequest(String),
   #[error("conflict: {0}")]
   Conflict(String),
   #[error("rate limited: {0}")]
   RateLimited(String),
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
   #[expect(
      clippy::cognitive_complexity,
      reason = "flat per-variant match with logging; splitting would obscure the mapping"
   )]
   fn into_response(self) -> Response {
      let (status, msg) = match self {
         Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_owned()),
         Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
         Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
         Self::RateLimited(message) => {
            return (
               StatusCode::TOO_MANY_REQUESTS,
               [(header::RETRY_AFTER, "3600")],
               Json(json!({"error": "rate_limited", "message": message})),
            )
               .into_response();
         },
         Self::Db(err) => {
            tracing::error!(error = ?err, "db error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "database error".to_owned(),
            )
         },
         Self::Pool(err) => {
            tracing::error!(error = ?err, "pool error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "database pool error".to_owned(),
            )
         },
         Self::Template(err) => {
            tracing::error!(error = ?err, "template render error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "template error".to_owned(),
            )
         },
         Self::Internal(err) => {
            tracing::error!(error = ?err, "internal error");
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "internal error".to_owned(),
            )
         },
      };
      (status, msg).into_response()
   }
}

pub type ApiResult<T> = Result<T, ApiError>;
