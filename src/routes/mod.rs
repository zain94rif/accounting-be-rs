pub mod health;
pub mod accounts;
pub mod journals;

use axum::Router;

pub fn router() -> Router<crate::app::AppState> {
    Router::new()
        .merge(health::router())
        .nest("/v1/accounts", accounts::router())
        .nest("/v1/journals", journals::router())
}
