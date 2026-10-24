//! Password hashing helpers using Argon2id.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use password_hash::Error as PasswordHashError;

use crate::error::{AppError, AppResult};

/// Hash a password with Argon2id (recommended parameters via default).
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!(error = %e, "password hashing failed");
            AppError::Internal
        })?
        .to_string();

    Ok(hash)
}

/// Verify a password against a stored PHC string.
pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(password_hash).map_err(|e| {
        tracing::error!(error = %e, "invalid password hash format");
        AppError::Internal
    })?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(PasswordHashError::Password) => Ok(false),
        Err(e) => {
            tracing::error!(error = %e, "password verification error");
            Err(AppError::Internal)
        }
    }
}
