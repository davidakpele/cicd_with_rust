-- Add migration script here
DROP TABLE IF EXISTS job_logs;
DROP TABLE IF EXISTS jobs;
DROP TABLE IF EXISTS repos;
DROP TYPE IF EXISTS job_status;

-- Enum for job status
CREATE TYPE job_status AS ENUM ('Pending', 'Running', 'Success', 'Failed');

-- Repo table
CREATE TABLE repos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    branch TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Job table
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
    commit_hash TEXT NOT NULL,
    status job_status DEFAULT 'Pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

-- Job logs table
CREATE TABLE job_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID REFERENCES jobs(id) ON DELETE CASCADE,
    line TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
