use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("not found")]
    NotFound,

    #[error("bad request: {0}")]
    BadRequest(String),
}

#[derive(Serialize)]
struct ErrBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::Db(err) => {
                if let Some(db_err) = err.as_database_error() {
                    // Cek unique violation Postgres (error code 23505)
                    if db_err.code().as_deref() == Some("23505") {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(ErrBody {
                                error: "Kode unik sudah terdaftar. Silakan gunakan kode lain.".to_string(),
                            }),
                        )
                            .into_response();
                    }
                }
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrBody {
                        error: self.to_string(),
                    }),
                )
                    .into_response()
            }
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrBody {
                    error: self.to_string(),
                }),
            )
                .into_response(),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrBody {
                    error: msg.clone(),
                }),
            )
                .into_response(),
        }
    }
}
