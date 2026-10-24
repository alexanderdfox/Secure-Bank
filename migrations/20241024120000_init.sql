-- Enable UUID generation if needed
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Users
CREATE TABLE users (
    id          UUID PRIMARY KEY,
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users (email);

-- Accounts
CREATE TYPE account_type AS ENUM ('checking', 'savings');
CREATE TYPE currency AS ENUM ('USD', 'EUR');

CREATE TABLE accounts (
    id            UUID PRIMARY KEY,
    owner_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_type  account_type NOT NULL,
    currency      currency NOT NULL,
    name          TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_accounts_owner ON accounts (owner_id);

-- Ledger entries (double-entry)
CREATE TYPE entry_direction AS ENUM ('debit', 'credit');

CREATE TABLE ledger_entries (
    id              UUID PRIMARY KEY,
    transaction_id  UUID NOT NULL,
    account_id      UUID NOT NULL REFERENCES accounts(id),
    direction       entry_direction NOT NULL,
    amount          NUMERIC(28, 8) NOT NULL CHECK (amount > 0),
    currency        currency NOT NULL,
    description     TEXT,
    initiated_by    UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ledger_account ON ledger_entries (account_id);
CREATE INDEX idx_ledger_tx ON ledger_entries (transaction_id);
CREATE INDEX idx_ledger_created ON ledger_entries (created_at);

-- Simple audit log for sensitive actions
CREATE TABLE audit_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID REFERENCES users(id),
    action      TEXT NOT NULL,
    resource    TEXT,
    details     JSONB,
    ip          TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user ON audit_log (user_id);
CREATE INDEX idx_audit_created ON audit_log (created_at);
