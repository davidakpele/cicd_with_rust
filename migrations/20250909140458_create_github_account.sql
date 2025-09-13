-- 2025_09_09_01_create_github_accounts.sql

CREATE TABLE github_accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL UNIQUE,
    email VARCHAR(255),
    username VARCHAR(255) NOT NULL UNIQUE,
    avatar_url TEXT,
    github_profile_url TEXT,
    github_repo_url TEXT,
    github_id BIGINT NOT NULL UNIQUE,
    github_token TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    last_login TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Optional: index on github_id for faster lookups
CREATE UNIQUE INDEX idx_github_accounts_github_id ON github_accounts (github_id);
