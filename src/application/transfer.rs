use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    AccountBalance, AccountId, Currency, EntryDirection, LedgerEntry, Money, TransactionId,
    Transfer, UserId,
};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub from_account_id: AccountId,
    pub to_account_id: AccountId,
    pub amount: Decimal,
    pub description: Option<String>,
}

/// Execute a double-entry transfer.
///
/// - Verifies ownership / existence of accounts
/// - Ensures same currency
/// - Checks sufficient balance on source
/// - Writes two balanced ledger entries in one transaction
pub async fn execute_transfer(
    pool: &PgPool,
    initiated_by: UserId,
    req: TransferRequest,
) -> AppResult<TransactionId> {
    if req.amount <= Decimal::ZERO {
        return Err(AppError::Validation("amount must be positive".into()));
    }
    if req.from_account_id == req.to_account_id {
        return Err(AppError::Validation("cannot transfer to the same account".into()));
    }

    let mut tx = pool.begin().await?;

    // Load both accounts and verify ownership
    let from = load_account(&mut tx, req.from_account_id).await?;
    let to = load_account(&mut tx, req.to_account_id).await?;

    if from.owner_id != initiated_by {
        return Err(AppError::Forbidden);
    }
    // For skeleton we allow transfers to any account; in real systems add more checks.

    if from.currency != to.currency {
        return Err(AppError::CurrencyMismatch);
    }

    let currency = from.currency;
    let money = Money::new(req.amount, currency);

    // Compute current balance of source account
    let balance = compute_balance(&mut tx, req.from_account_id, currency).await?;
    if balance.amount < req.amount {
        return Err(AppError::InsufficientFunds);
    }

    let transaction_id = TransactionId::new();
    let transfer = Transfer {
        transaction_id,
        from_account: req.from_account_id,
        to_account: req.to_account_id,
        amount: money,
        description: req.description.clone(),
        initiated_by,
        created_at: chrono::Utc::now(),
    };

    let (debit, credit) = transfer.to_entries();

    insert_entry(&mut tx, &debit).await?;
    insert_entry(&mut tx, &credit).await?;

    // Audit
    sqlx::query(
        r#"
        INSERT INTO audit_log (user_id, action, resource, details)
        VALUES ($1, 'transfer.execute', $2, $3)
        "#,
    )
    .bind(initiated_by.0)
    .bind(format!("tx:{}", transaction_id.0))
    .bind(serde_json::json!({
        "from": req.from_account_id.0,
        "to": req.to_account_id.0,
        "amount": req.amount.to_string(),
        "currency": currency.code(),
    }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(transaction_id)
}

async fn load_account(
    tx: &mut Transaction<'_, Postgres>,
    id: AccountId,
) -> AppResult<crate::domain::Account> {
    let row: Option<(Uuid, Uuid, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            r#"
            SELECT id, owner_id, account_type::text, currency::text, name, created_at, updated_at
            FROM accounts WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&mut **tx)
        .await?;

    let Some((id, owner_id, account_type, currency, name, created_at, updated_at)) = row else {
        return Err(AppError::NotFound(format!("account {}", id)));
    };

    let account_type = match account_type.as_str() {
        "checking" => crate::domain::AccountType::Checking,
        "savings" => crate::domain::AccountType::Savings,
        _ => return Err(AppError::Internal),
    };
    let currency = match currency.as_str() {
        "USD" => Currency::Usd,
        "EUR" => Currency::Eur,
        _ => return Err(AppError::Internal),
    };

    Ok(crate::domain::Account {
        id: AccountId(id),
        owner_id: UserId(owner_id),
        account_type,
        currency,
        name,
        created_at,
        updated_at,
    })
}

async fn compute_balance(
    tx: &mut Transaction<'_, Postgres>,
    account_id: AccountId,
    currency: Currency,
) -> AppResult<Money> {
    // Sum credits - sum debits
    let row: (Option<Decimal>, Option<Decimal>) = sqlx::query_as(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN direction = 'credit' THEN amount ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN direction = 'debit' THEN amount ELSE 0 END), 0)
        FROM ledger_entries
        WHERE account_id = $1 AND currency = $2
        "#,
    )
    .bind(account_id.0)
    .bind(currency.code())
    .fetch_one(&mut **tx)
    .await?;

    let credits = row.0.unwrap_or(Decimal::ZERO);
    let debits = row.1.unwrap_or(Decimal::ZERO);
    let balance = credits - debits;

    Ok(Money::new(balance, currency))
}

async fn insert_entry(
    tx: &mut Transaction<'_, Postgres>,
    entry: &LedgerEntry,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO ledger_entries
            (id, transaction_id, account_id, direction, amount, currency, description, initiated_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(entry.id.0)
    .bind(entry.transaction_id.0)
    .bind(entry.account_id.0)
    .bind(match entry.direction {
        EntryDirection::Debit => "debit",
        EntryDirection::Credit => "credit",
    })
    .bind(entry.amount.amount)
    .bind(entry.amount.currency.code())
    .bind(&entry.description)
    .bind(entry.initiated_by.0)
    .bind(entry.created_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn get_balance(pool: &PgPool, account_id: AccountId) -> AppResult<AccountBalance> {
    // Load currency first
    let currency_row: Option<(String,)> = sqlx::query_as(
        "SELECT currency::text FROM accounts WHERE id = $1",
    )
    .bind(account_id.0)
    .fetch_optional(pool)
    .await?;

    let Some((currency_str,)) = currency_row else {
        return Err(AppError::NotFound(format!("account {}", account_id)));
    };

    let currency = match currency_str.as_str() {
        "USD" => Currency::Usd,
        "EUR" => Currency::Eur,
        _ => return Err(AppError::Internal),
    };

    let mut tx = pool.begin().await?;
    let balance = compute_balance(&mut tx, account_id, currency).await?;
    tx.commit().await?;

    Ok(AccountBalance {
        account_id,
        currency,
        balance,
    })
}
