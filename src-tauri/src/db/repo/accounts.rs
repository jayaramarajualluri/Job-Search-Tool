//! Accounts repository.  Password values are NEVER written through this
//! module — only the `credential_key` (OS keychain handle) and the
//! `has_saved_password` flag.

use crate::db::DbConn;
use crate::domain::account::{Account, PortalType};
use crate::error::AppResult;
use rusqlite::{params, OptionalExtension, Row};

pub fn list(c: &DbConn, company_id: Option<i64>) -> AppResult<Vec<Account>> {
    let rows: Vec<Account> = if let Some(cid) = company_id {
        let mut stmt = c.prepare(&format!("{} WHERE company_id = ?1 ORDER BY id", SELECT_BASE))?;
        stmt.query_map([cid], map_row)?.collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        let mut stmt = c.prepare(&format!("{} ORDER BY id", SELECT_BASE))?;
        stmt.query_map([], map_row)?.collect::<rusqlite::Result<Vec<_>>>()?
    };
    Ok(rows)
}

pub fn get(c: &DbConn, id: i64) -> AppResult<Option<Account>> {
    Ok(c.query_row(
        &format!("{} WHERE id = ?1", SELECT_BASE),
        [id],
        map_row,
    )
    .optional()?)
}

pub fn create(
    c: &DbConn,
    company_id: i64,
    portal_type: PortalType,
    login_url: Option<&str>,
    username: Option<&str>,
    requires_2fa: bool,
    notes: Option<&str>,
) -> AppResult<i64> {
    c.execute(
        "INSERT INTO accounts(company_id, portal_type, login_url, username,
            requires_2fa, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            company_id,
            portal_type_str(portal_type),
            login_url,
            username,
            if requires_2fa { 1 } else { 0 },
            notes,
        ],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn set_credential_key(c: &DbConn, id: i64, key: Option<&str>) -> AppResult<()> {
    let saved = if key.is_some() { 1 } else { 0 };
    c.execute(
        "UPDATE accounts SET credential_key = ?1, has_saved_password = ?2,
            updated_at = datetime('now') WHERE id = ?3",
        params![key, saved, id],
    )?;
    Ok(())
}

pub fn update(
    c: &DbConn,
    id: i64,
    login_url: Option<&str>,
    username: Option<&str>,
    requires_2fa: bool,
    notes: Option<&str>,
) -> AppResult<()> {
    c.execute(
        "UPDATE accounts SET login_url = ?1, username = ?2, requires_2fa = ?3,
            notes = ?4, updated_at = datetime('now') WHERE id = ?5",
        params![login_url, username, if requires_2fa { 1 } else { 0 }, notes, id],
    )?;
    Ok(())
}

pub fn mark_used(c: &DbConn, id: i64) -> AppResult<()> {
    c.execute(
        "UPDATE accounts SET last_used_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

pub fn delete(c: &DbConn, id: i64) -> AppResult<()> {
    c.execute("DELETE FROM accounts WHERE id = ?1", [id])?;
    Ok(())
}

// ---------------------------------------------------------------------------

fn portal_type_str(p: PortalType) -> &'static str {
    match p {
        PortalType::Workday => "workday",
        PortalType::Greenhouse => "greenhouse",
        PortalType::Lever => "lever",
        PortalType::Icims => "icims",
        PortalType::Smartrecruiters => "smartrecruiters",
        PortalType::Ashby => "ashby",
        PortalType::OracleTaleo => "oracle_taleo",
        PortalType::Successfactors => "successfactors",
        PortalType::Custom => "custom",
        PortalType::Unknown => "unknown",
    }
}
fn parse_portal(s: &str) -> PortalType {
    match s {
        "workday" => PortalType::Workday,
        "greenhouse" => PortalType::Greenhouse,
        "lever" => PortalType::Lever,
        "icims" => PortalType::Icims,
        "smartrecruiters" => PortalType::Smartrecruiters,
        "ashby" => PortalType::Ashby,
        "oracle_taleo" => PortalType::OracleTaleo,
        "successfactors" => PortalType::Successfactors,
        "custom" => PortalType::Custom,
        _ => PortalType::Unknown,
    }
}

const SELECT_BASE: &str = "SELECT id, company_id, portal_type, login_url,
    username, credential_key, has_saved_password, requires_2fa, notes,
    last_used_at, created_at, updated_at FROM accounts";

fn map_row(r: &Row<'_>) -> rusqlite::Result<Account> {
    Ok(Account {
        id: r.get(0)?,
        company_id: r.get(1)?,
        portal_type: parse_portal(&r.get::<_, String>(2)?),
        login_url: r.get(3)?,
        username: r.get(4)?,
        credential_key: r.get(5)?,
        has_saved_password: r.get::<_, i64>(6)? != 0,
        requires_2fa: r.get::<_, i64>(7)? != 0,
        notes: r.get(8)?,
        last_used_at: r.get(9)?,
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
    })
}
