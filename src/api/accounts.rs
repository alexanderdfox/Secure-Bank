use axum::{
    extract::{Path, State},
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::application::{execute_transfer, get_balance, TransferRequest};
use crate::domain::{Account, AccountId, AccountType, Currency, UserId};
use crate::error::{AppError, AppResult};

/// Very simple auth extractor for the skeleton.
/// Client sends `X-User-Id: <uuid>` after login/register.
/// Replace with proper session / JWT middleware later.
pub async fn extract_user_id(
    headers: axum::http::HeaderMap,
) -> AppResult<UserId> {
    let header = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(header).map_err(|_| AppError::Unauthorized)?;
    Ok(UserId(uuid))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountBody {
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: AccountId,
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
}

pub async fn create_account(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateAccountBody>,
) -> AppResult<Json<AccountResponse>> {
    let user_id = extract_user_id(headers).await?;

    if body.name.trim().is_empty() {
        return Err(AppError::Validation("name required".into()));
    }

    let account = Account::new(user_id, body.account_type, body.currency, body.name.trim().to_string());

    sqlx::query(
        r#"
        INSERT INTO accounts (id, owner_id, account_type, currency, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(account.id.0)
    .bind(account.owner_id.0)
    .bind(match account.account_type {
        AccountType::Checking => "checking",
        AccountType::Savings => "savings",
    })
    .bind(account.currency.code())
    .bind(&account.name)
    .bind(account.created_at)
    .bind(account.updated_at)
    .execute(&pool)
    .await?;

    Ok(Json(AccountResponse {
        id: account.id,
        name: account.name,
        account_type: account.account_type,
        currency: account.currency,
    }))
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub account_id: AccountId,
    pub currency: Currency,
    pub balance: String, // decimal as string for JSON safety
}

pub async fn get_account_balance(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Path(account_id): Path<Uuid>,
) -> AppResult<Json<BalanceResponse>> {
    let user_id = extract_user_id(headers).await?;
    let account_id = AccountId(account_id);

    // Ownership check is done inside get_balance path via load; for skeleton we also verify owner.
    let bal = get_balance(&pool, account_id).await?;

    // Extra ownership check
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT owner_id FROM accounts WHERE id = $1")
        .bind(account_id.0)
        .fetch_optional(&pool)
        .await?;

    match owner {
        Some((oid,)) if oid == user_id.0 => {}
        _ => return Err(AppError::Forbidden),
    }

    Ok(Json(BalanceResponse {
        account_id: bal.account_id,
        currency: bal.currency,
        balance: bal.balance.amount.to_string(),
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct TransferBody {
    pub from_account_id: Uuid,
    pub to_account_id: Uuid,
    pub amount: Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub transaction_id: Uuid,
}

pub async fn transfer_handler(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(body): Json<TransferBody>,
) -> AppResult<Json<TransferResponse>> {
    let user_id = extract_user_id(headers).await?;

    let tx_id = execute_transfer(
        &pool,
        user_id,
        TransferRequest {
            from_account_id: AccountId(body.from_account_id),
            to_account_id: AccountId(body.to_account_id),
            amount: body.amount,
            description: body.description,
        },
    )
    .await?;

    Ok(Json(TransferResponse {
        transaction_id: tx_id.0,
    }))
}
