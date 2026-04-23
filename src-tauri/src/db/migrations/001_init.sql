-- Migration 001: initial schema.
-- SQLite.  All timestamps are UTC ISO-8601 strings; date-only columns are
-- YYYY-MM-DD.  We use TEXT for enums and validate in code + CHECK clauses
-- so we can extend vocabularies without a migration.

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

-- ---------------------------------------------------------------------------
-- schema_migrations — applied-version ledger.  Migrations are applied in
-- ascending numeric order; each one runs inside a transaction.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS schema_migrations (
    version     INTEGER PRIMARY KEY,
    applied_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ---------------------------------------------------------------------------
-- companies
-- ---------------------------------------------------------------------------
CREATE TABLE companies (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT    NOT NULL,
    normalized_name     TEXT    NOT NULL UNIQUE,
    company_folder_path TEXT,
    notes               TEXT,
    created_at          TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ---------------------------------------------------------------------------
-- jobs
--
-- sponsorship_confidence: one of
--   explicit_sponsor | sponsor_likely | sponsor_unclear | sponsor_unlikely
--   | explicit_no_sponsorship
-- work_mode: remote | hybrid | onsite | unknown
-- recency_bucket: today | yesterday | week | two_weeks | month | older
-- status: new | resume_prepared | ready_to_apply | applied | oa_received
--         | interview | rejected | closed | skipped
-- ---------------------------------------------------------------------------
CREATE TABLE jobs (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id              INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,

    source_name             TEXT    NOT NULL,          -- greenhouse | lever | ashby | indeed | linkedin | manual | employer_portal
    source_url              TEXT,
    apply_url               TEXT,                       -- canonical employer/ATS URL preferred
    role_title              TEXT    NOT NULL,
    normalized_role_title   TEXT    NOT NULL,
    job_external_id         TEXT,                       -- ATS posting id if present

    location                TEXT,
    state_or_region         TEXT,                       -- 2-letter US state, or region code, or NULL
    work_mode               TEXT    NOT NULL DEFAULT 'unknown'
        CHECK (work_mode IN ('remote','hybrid','onsite','unknown')),

    posted_date             TEXT,                       -- YYYY-MM-DD
    recency_bucket          TEXT
        CHECK (recency_bucket IN ('today','yesterday','week','two_weeks','month','older')),

    jd_text                 TEXT,
    jd_summary              TEXT,

    sponsorship_confidence  TEXT    NOT NULL DEFAULT 'sponsor_unclear'
        CHECK (sponsorship_confidence IN (
            'explicit_sponsor','sponsor_likely','sponsor_unclear',
            'sponsor_unlikely','explicit_no_sponsorship'
        )),
    sponsorship_reason      TEXT,

    match_score             REAL,                       -- 0..100
    match_explanation       TEXT,                       -- JSON blob describing contributions

    role_folder_path        TEXT,
    jd_file_path            TEXT,
    resume_file_path        TEXT,
    cover_letter_file_path  TEXT,

    status                  TEXT    NOT NULL DEFAULT 'new'
        CHECK (status IN (
            'new','resume_prepared','ready_to_apply','applied',
            'oa_received','interview','rejected','closed','skipped'
        )),

    created_at              TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Dedupe on (company, external id) when present; partial unique index.
CREATE UNIQUE INDEX idx_jobs_ext_id
    ON jobs(company_id, job_external_id)
    WHERE job_external_id IS NOT NULL;

CREATE INDEX idx_jobs_posted_date   ON jobs(posted_date DESC);
CREATE INDEX idx_jobs_status        ON jobs(status);
CREATE INDEX idx_jobs_recency       ON jobs(recency_bucket);
CREATE INDEX idx_jobs_company       ON jobs(company_id);
CREATE INDEX idx_jobs_match_score   ON jobs(match_score DESC);

-- ---------------------------------------------------------------------------
-- accounts — employer portal logins.  NEVER store password content.
-- `credential_key` is the key used to look up the password in the OS
-- keychain.  If a password has not been saved, `has_saved_password = 0`.
-- ---------------------------------------------------------------------------
CREATE TABLE accounts (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id           INTEGER NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    portal_type          TEXT    NOT NULL DEFAULT 'unknown'
        CHECK (portal_type IN (
            'workday','greenhouse','lever','icims','smartrecruiters',
            'ashby','oracle_taleo','successfactors','custom','unknown'
        )),
    login_url            TEXT,
    username             TEXT,
    credential_key       TEXT,        -- keyring lookup key; opaque
    has_saved_password   INTEGER NOT NULL DEFAULT 0 CHECK (has_saved_password IN (0,1)),
    requires_2fa         INTEGER NOT NULL DEFAULT 0 CHECK (requires_2fa IN (0,1)),
    notes                TEXT,
    last_used_at         TEXT,
    created_at           TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_accounts_company ON accounts(company_id);

-- ---------------------------------------------------------------------------
-- resumes — one row per tailored variant.
-- reuse_strategy: new | reused | edited_reuse
-- ---------------------------------------------------------------------------
CREATE TABLE resumes (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id               INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    base_resume_name     TEXT    NOT NULL,
    resume_variant_name  TEXT    NOT NULL,
    source_resume_path   TEXT,
    output_resume_path   TEXT,
    reuse_strategy       TEXT    NOT NULL DEFAULT 'new'
        CHECK (reuse_strategy IN ('new','reused','edited_reuse')),
    similarity_score     REAL,                    -- 0..1 against prior variant
    created_at           TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_resumes_job ON resumes(job_id);

-- ---------------------------------------------------------------------------
-- settings — key/value store.  `value` is a JSON string.
-- ---------------------------------------------------------------------------
CREATE TABLE settings (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    key         TEXT    NOT NULL UNIQUE,
    value       TEXT    NOT NULL,
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- ---------------------------------------------------------------------------
-- ingestion_runs — one row per ingestion cycle.  `source_summary` is a JSON
-- map of source_name → {fetched, filtered, prepared, errors}.
-- ---------------------------------------------------------------------------
CREATE TABLE ingestion_runs (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at       TEXT    NOT NULL,
    ended_at         TEXT,
    source_summary   TEXT,                            -- JSON
    total_fetched    INTEGER NOT NULL DEFAULT 0,
    total_filtered   INTEGER NOT NULL DEFAULT 0,
    total_prepared   INTEGER NOT NULL DEFAULT 0,
    notes            TEXT
);

CREATE INDEX idx_ingestion_runs_started ON ingestion_runs(started_at DESC);

-- ---------------------------------------------------------------------------
-- job_skills — extracted skills per job.  kind = required | preferred | nice_to_have
-- ---------------------------------------------------------------------------
CREATE TABLE job_skills (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id  INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    skill   TEXT    NOT NULL,
    kind    TEXT    NOT NULL DEFAULT 'required'
        CHECK (kind IN ('required','preferred','nice_to_have')),
    UNIQUE(job_id, skill, kind)
);

CREATE INDEX idx_job_skills_skill ON job_skills(skill);

-- ---------------------------------------------------------------------------
-- skill_aliases — user-editable synonyms.  canonical should be lowercase.
-- ---------------------------------------------------------------------------
CREATE TABLE skill_aliases (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    canonical  TEXT NOT NULL,
    alias      TEXT NOT NULL,
    UNIQUE(canonical, alias)
);

CREATE INDEX idx_skill_aliases_alias ON skill_aliases(alias);

-- ---------------------------------------------------------------------------
-- status_history — append-only audit trail of status changes.
-- ---------------------------------------------------------------------------
CREATE TABLE status_history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id       INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    from_status  TEXT,
    to_status    TEXT NOT NULL,
    at           TEXT NOT NULL DEFAULT (datetime('now')),
    notes        TEXT
);

CREATE INDEX idx_status_history_job ON status_history(job_id);
