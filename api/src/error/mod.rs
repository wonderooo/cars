use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("diesel error: `{0}`")]
    Diesel(#[from] diesel::result::Error),
    #[error("postgres pool error: `{0}`")]
    PgPool(#[from] diesel_async::pooled_connection::deadpool::PoolError),
    #[error("lot vehicle with given ln not found: `{0}`")]
    LotVehicleNotFound(i32),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::LotVehicleNotFound(ln) => (
                StatusCode::NOT_FOUND,
                format!("lot vehicle with lot number not found: `{}`", ln),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_string(),
            ),
        };

        (status, ApiJson(ErrorResponse { message })).into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ErrorResponse {
    message: String,
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
struct ApiJson<T>(T);

impl<T> IntoResponse for ApiJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
