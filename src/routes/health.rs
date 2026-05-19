use axum::{Router, routing::get};

pub fn router() -> Router<crate::app::AppState> {
    Router::new().route("/health", get(|| async { "ok" }))
}
