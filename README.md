# Secure Bank (Rust Skeleton)

**Educational / foundation project only.**

This is a deliberately conservative starting point for a banking-style application written in Rust.  
It demonstrates:

- Memory-safe domain modeling
- Double-entry ledger fundamentals
- Secure password hashing (Argon2id)
- Typed money handling
- Axum + SQLx + PostgreSQL architecture
- Clear separation of domain / application / infrastructure / API layers

## ⚠️ Critical Disclaimer

This code is **not** production-ready for handling real customer funds.

Real-world banking systems require:

- Regulatory licenses / partnerships
- Independent security audits & penetration testing
- SOC 2 / PCI DSS / GLBA / FFIEC compliance work
- Professional key management & HSM usage
- Fraud detection, AML/KYC systems
- Operational maturity, incident response, insurance
- Ongoing monitoring and formal threat modeling

Use this as a learning foundation and architectural reference only.

## Features (Current Skeleton)

- User registration & login with Argon2id
- Account creation
- Double-entry transfer between accounts (same user or internal)
- Balance derived from ledger
- Basic audit logging of sensitive actions
- Request tracing & structured logging

## Tech Stack

- **Axum** – HTTP framework
- **SQLx** – compile-time checked SQL + migrations
- **PostgreSQL** – primary store
- **Argon2id** – password hashing
- **rust_decimal** – precise money
- **Tokio** – async runtime
- **tracing** – observability

## Project Layout

```
src/
├── domain/          # Pure business types & invariants (no I/O)
├── application/     # Use cases / orchestration
├── infrastructure/  # DB, crypto helpers, external services
├── api/             # HTTP handlers & routing
├── error/           # Unified error type
├── config.rs
├── lib.rs
└── main.rs
```

## Getting Started

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- `sqlx-cli` (`cargo install sqlx-cli --features postgres`)

### Setup

```bash
cp .env.example .env
# Edit DATABASE_URL and other secrets

# Create database and run migrations
sqlx database create
sqlx migrate run

# Run the server
cargo run
```

Default listen address: `http://127.0.0.1:3000`

### Environment Variables

See `.env.example`.

## Security Notes (Current Implementation)

- Passwords stored as Argon2id PHC strings
- No plaintext secrets in logs
- Parameterized queries only
- Money represented in minor units via `rust_decimal`
- Transfers are transactional

Still missing for anything near production:

- MFA / WebAuthn
- Session / token revocation & rotation
- Rate limiting per principal
- Proper secrets management (Vault / cloud KMS)
- Encryption of sensitive columns at rest
- Comprehensive audit immutability
- Fraud / velocity rules
- Formal authorization model beyond simple ownership

## License

MIT OR Apache-2.0
