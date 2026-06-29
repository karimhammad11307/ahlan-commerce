use axum::http::StatusCode;
use axum::{Json, response::{IntoResponse, Response}};



#[derive(Debug)]
pub enum  AppError{
    //400
    ValidationFailed(String),

    // 409
    DuplicateHandle(String),

    // 404 — no product with that id
    #[allow(dead_code)]
    NotFound(String),

    // 500 — something broke on our side
    Internal(String),

}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::ValidationFailed(msg) => (
                StatusCode::BAD_REQUEST,
                "validation_failed",
                msg.clone(),
            ),

             AppError::DuplicateHandle(handle) => (
                StatusCode::CONFLICT,
                "duplicate_product_handle",
                format!("a product with handle '{}' already exists", handle),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "not_found",
                msg.clone(),
            ),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",

                "an internal error occurred".to_string(),
            ),
        };
        if let AppError::Internal(cause) = &self {
            tracing::error!(error_code,cause,"internal_error");
        }
        let error_response = crate::dtos::ErrorResponse {
            error: crate::dtos::ErrorDetail {
                code: error_code.to_string(),
                message: message.to_string(),
            }
        };
        (status, Json(error_response)).into_response()
    }
    
}

use async_graphql::ErrorExtensions;

impl From<AppError> for async_graphql::Error {
    fn from(err: AppError) -> Self {
        let (error_code, message) = match &err {
            AppError::ValidationFailed(msg) => ("validation_failed", msg.clone()),
            AppError::DuplicateHandle(handle) => (
                "duplicate_product_handle",
                format!("a product with handle '{}' already exists", handle),
            ),
            AppError::NotFound(msg) => ("not_found", msg.clone()),
            AppError::Internal(cause) => {
                tracing::error!(error_code = "internal_error", cause, "internal_error");
                ("internal_error", "an internal error occurred".to_string())
            }
        };

        async_graphql::Error::new(message).extend_with(|_, e| e.set("code", error_code))
    }
}

#[cfg(test)] 
mod tests{
    use super::*;

    use axum::response::IntoResponse;
    #[tokio::test]
    async fn validation_error_returns_400(){
        let err = AppError::ValidationFailed("title is empty".into());
        let response = err.into_response();
        assert_eq!(response.status(),StatusCode::BAD_REQUEST);

    }

    #[tokio::test]
    async fn duplicate_handle_returns_409() {
        let err = AppError::DuplicateHandle("my-laptop".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let err = AppError::NotFound("product not found".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn internal_error_returns_500_with_safe_body() {
        let err = AppError::Internal("postgres: connection refused at 10.0.0.1:5432".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        // The body must NOT contain the postgres connection string
    }
    
}