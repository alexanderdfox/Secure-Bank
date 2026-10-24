use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::money::Currency;
use super::user::UserId;

/// Strongly typed account identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccountId(pub Uuid);

impl AccountId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for AccountId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Checking,
    Savings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub owner_id: UserId,
    pub account_type: AccountType,
    pub currency: Currency,
    /// Human-readable label (e.g. "Primary Checking")
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(owner_id: UserId, account_type: AccountType, currency: Currency, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: AccountId::new(),
            owner_id,
            account_type,
            currency,
            name,
            created_at: now,
            updated_at: now,
        }
    }
}
