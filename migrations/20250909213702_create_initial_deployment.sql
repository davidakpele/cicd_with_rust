-- Add migration script here
CREATE TABLE deployments (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    repo_name TEXT NOT NULL,
    branch TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    deployment_id BIGINT REFERENCES deployments(id),
    port INT UNIQUE, -- nullable
    in_use BOOLEAN DEFAULT TRUE,
    logs TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
