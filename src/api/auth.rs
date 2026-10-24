use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

use crate::application::{login, register, LoginRequest, RegisterRequest};
use crate::domain::UserId;
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterBody {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12, message = "password must be at least 12 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: UserId,
    pub email: String,
    /// Temporary: in a real system this would be a session token / JWT.
    /// For the skeleton we return the user_id so the client can send it as
    /// `X-User-Id` header for subsequent requests.
    pub token: String,
}

pub async fn register_handler(
    State(pool): State<PgPool>,
    Json(body): Json<RegisterBody>,
) -> AppResult<Json<AuthResponse>> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = register(
        &pool,
        RegisterRequest {
            email: body.email,
            password: body.password,
        },
    )
    .await?;

    Ok(Json(AuthResponse {
        user_id: user.id,
        email: user.email,
        token: user.id.0.to_string(),
    }))
}

pub async fn login_handler(
    State(pool): State<PgPool>,
    Json(body): Json<LoginBody>,
) -> AppResult<Json<AuthResponse>> {
    let user = login(
        &pool,
        LoginRequest {
            email: body.email,
            password: body.password,
        },
    )
    .await?;

    Ok(Json(AuthResponse {
        user_id: user.id,
        email: user.email,
        token: user.id.0.to_string(),
    }))
}
