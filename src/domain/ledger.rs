//! Double-entry ledger primitives.
//!
//! Every transfer creates two balanced entries (debit + credit).
//! Account balances are derived by summing entries, never mutated in isolation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::account::AccountId;
use super::money::{Currency, Money};
use super::user::UserId;

/// Unique identifier for a ledger entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntryId(pub Uuid);

impl EntryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Groups the two (or more) entries that constitute a single economic event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryDirection {
    Debit,
    Credit,
}

/// A single ledger entry. Always part of a balanced transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: EntryId,
    pub transaction_id: TransactionId,
    pub account_id: AccountId,
    pub direction: EntryDirection,
    pub amount: Money,
    /// Optional human-readable description
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Who initiated the economic event (for audit)
    pub initiated_by: UserId,
}

impl LedgerEntry {
    pub fn debit(
        transaction_id: TransactionId,
        account_id: AccountId,
        amount: Money,
        description: Option<String>,
        initiated_by: UserId,
    ) -> Self {
        Self {
            id: EntryId::new(),
            transaction_id,
            account_id,
            direction: EntryDirection::Debit,
            amount,
            description,
            created_at: Utc::now(),
            initiated_by,
        }
    }

    pub fn credit(
        transaction_id: TransactionId,
        account_id: AccountId,
        amount: Money,
        description: Option<String>,
        initiated_by: UserId,
    ) -> Self {
        Self {
            id: EntryId::new(),
            transaction_id,
            account_id,
            direction: EntryDirection::Credit,
            amount,
            description,
            created_at: Utc::now(),
            initiated_by,
        }
    }
}

/// A balanced pair of entries representing a transfer.
#[derive(Debug, Clone)]
pub struct Transfer {
    pub transaction_id: TransactionId,
    pub from_account: AccountId,
    pub to_account: AccountId,
    pub amount: Money,
    pub description: Option<String>,
    pub initiated_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl Transfer {
    /// Create the two balanced ledger entries for this transfer.
    /// Debit the source, credit the destination.
    pub fn to_entries(&self) -> (LedgerEntry, LedgerEntry) {
        let debit = LedgerEntry::debit(
            self.transaction_id,
            self.from_account,
            self.amount,
            self.description.clone(),
            self.initiated_by,
        );
        let credit = LedgerEntry::credit(
            self.transaction_id,
            self.to_account,
            self.amount,
            self.description.clone(),
            self.initiated_by,
        );
        (debit, credit)
    }
}

/// Result of computing an account balance from the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account_id: AccountId,
    pub currency: Currency,
    pub balance: Money,
}
