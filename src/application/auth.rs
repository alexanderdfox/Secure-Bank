use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{User, UserId};
use crate::error::{AppError, AppResult};
use crate::infrastructure::{hash_password, verify_password};

#[derive(Debug, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(pool: &PgPool, req: RegisterRequest) -> AppResult<User> {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::Validation("invalid email".into()));
    }
    if req.password.len() < 12 {
        return Err(AppError::Validation(
            "password must be at least 12 characters".into(),
        ));
    }

    // Check uniqueness
    let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await?;

    if exists.is_some() {
        return Err(AppError::Conflict("email already registered".into()));
    }

    let password_hash = hash_password(&req.password)?;
    let user = User::new(email, password_hash);

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user.id.0)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(user.created_at)
    .bind(user.updated_at)
    .execute(pool)
    .await?;

    // Audit
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (user_id, action, resource, details)
        VALUES ($1, 'user.register', $2, $3)
        "#,
    )
    .bind(user.id.0)
    .bind(format!("user:{}", user.id))
    .bind(serde_json::json!({ "email": user.email }))
    .execute(pool)
    .await;

    Ok(user)
}

pub async fn login(pool: &PgPool, req: LoginRequest) -> AppResult<User> {
    let email = req.email.trim().to_lowercase();

    let row: Option<(Uuid, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users WHERE email = $1
            "#,
        )
        .bind(&email)
        .fetch_optional(pool)
        .await?;

    let Some((id, email, password_hash, created_at, updated_at)) = row else {
        // Constant-time-ish response: still verify a dummy hash? For skeleton we just fail.
        return Err(AppError::Unauthorized);
    };

    let valid = verify_password(&req.password, &password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let user = User {
        id: UserId(id),
        email,
        password_hash,
        created_at,
        updated_at,
    };

    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (user_id, action, resource)
        VALUES ($1, 'user.login', $2)
        "#,
    )
    .bind(user.id.0)
    .bind(format!("user:{}", user.id))
    .execute(pool)
    .await;

    Ok(user)
}
