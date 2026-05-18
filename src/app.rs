use axum::Router;
use sqlx::PgPool;

use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn build_app(db: PgPool) -> Router {
    let state = AppState { db };

    Router::new()
        .merge(routes::router())
        .with_state(state)
}
