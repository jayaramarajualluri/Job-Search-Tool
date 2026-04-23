//! SQLite database: connection pool, migrations, and repository access.
//!
//! The DB file lives at `<app_data_dir>/jobsearch.sqlite` and is opened with
//! WAL journaling + foreign keys on.  Repositories are thin wrappers that
//! own no state; they borrow a pooled connection per call.

use crate::error::AppResult;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub mod repo;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConn = r2d2::PooledConnection<SqliteConnectionManager>;

/// Tauri-managed handle: cloneable, cheap, pool internally reference-counted.
#[derive(Clone)]
pub struct Database {
    pool: DbPool,
    #[allow(dead_code)]
    path: PathBuf,
}

impl Database {
    /// Open (or create) the SQLite file at `<data_dir>/jobsearch.sqlite`.
    pub fn open(data_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(data_dir)?;
        let path = data_dir.join("jobsearch.sqlite");
        let manager = SqliteConnectionManager::file(&path).with_init(|c| {
            c.execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA busy_timeout = 5000;",
            )
        });
        let pool = Pool::builder().max_size(8).build(manager)?;
        Ok(Self { pool, path })
    }

    pub fn conn(&self) -> AppResult<DbConn> {
        Ok(self.pool.get()?)
    }

    /// Apply any unapplied migrations in order.  Each migration runs in a
    /// transaction and writes to `schema_migrations` on success.
    pub fn migrate(&self) -> AppResult<()> {
        let mut c = self.conn()?;
        // Bootstrap the ledger so we can query applied versions.
        c.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                 version    INTEGER PRIMARY KEY,
                 applied_at TEXT NOT NULL DEFAULT (datetime('now'))
             );",
        )?;

        for (version, sql) in MIGRATIONS {
            if is_applied(&c, *version)? {
                continue;
            }
            let tx = c.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(version) VALUES (?1)",
                [version],
            )?;
            tx.commit()?;
            tracing::info!(version, "migration applied");
        }
        Ok(())
    }
}

fn is_applied(c: &Connection, v: i64) -> AppResult<bool> {
    let n: i64 = c.query_row(
        "SELECT COUNT(1) FROM schema_migrations WHERE version = ?1",
        [v],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

/// Ordered list of (version, SQL).  Add new entries; never edit existing.
const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("migrations/001_init.sql"))];
