//! Precise money handling.
//!
//! We store amounts as `Decimal` in the smallest currency unit (e.g. cents for USD)
//! and never use floating point for financial calculations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Neg, Sub};

/// ISO 4217 currency code (simplified for the skeleton).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Usd,
    Eur,
    // Extend as needed
}

impl Currency {
    pub fn code(self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
        }
    }

    pub fn decimals(self) -> u32 {
        match self {
            Currency::Usd | Currency::Eur => 2,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Money is always associated with a currency and uses fixed-point arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// Amount in minor units (e.g. cents). Can be negative for debits in ledger context.
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.amount > Decimal::ZERO
    }

    pub fn is_negative(&self) -> bool {
        self.amount < Decimal::ZERO
    }

    /// Checked addition that enforces same currency.
    pub fn checked_add(self, other: Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    pub fn checked_sub(self, other: Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}

impl Add for Money {
    type Output = Result<Money, MoneyError>;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for Money {
    type Output = Result<Money, MoneyError>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Neg for Money {
    type Output = Money;

    fn neg(self) -> Self::Output {
        Money {
            amount: -self.amount,
            currency: self.currency,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoneyError {
    #[error("currency mismatch: {left} vs {right}")]
    CurrencyMismatch { left: Currency, right: Currency },
}
