use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use super::accounts::{create_account, get_account_balance, transfer_handler};
use super::auth::{login_handler, register_handler};

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/accounts", post(create_account))
        .route("/accounts/:id/balance", get(get_account_balance))
        .route("/transfers", post(transfer_handler))
        .with_state(pool)
}
