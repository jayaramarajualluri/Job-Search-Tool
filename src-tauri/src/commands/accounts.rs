//! Account / employer-portal login commands.
//!
//! Password flow (strict):
//!   - `create_account` optionally takes a password and writes it to the OS
//!     keychain.  The DB row stores only the credential_key + the
//!     has_saved_password flag.
//!   - `save_account_password` is a dedicated path for updating a password
//!     without modifying other fields.
//!   - `reveal_account_password` returns the stored password on demand so
//!     the user can paste it into a portal; never persisted to the UI.
//!   - `delete_account` also deletes the keychain entry.

use crate::db::repo;
use crate::db::Database;
use crate::domain::account::{Account, AccountInput};
use crate::error::AppResult;
use crate::secrets::keyring;
use tauri::State;

#[tauri::command]
pub fn list_accounts(
    db: State<'_, Database>,
    company_id: Option<i64>,
) -> AppResult<Vec<Account>> {
    let c = db.conn()?;
    repo::accounts::list(&c, company_id)
}

#[tauri::command]
pub fn create_account(
    db: State<'_, Database>,
    input: AccountInput,
) -> AppResult<Account> {
    let id = {
        let c = db.conn()?;
        let id = repo::accounts::create(
            &c,
            input.company_id,
            input.portal_type,
            input.login_url.as_deref(),
            input.username.as_deref(),
            input.requires_2fa,
            input.notes.as_deref(),
        )?;
        id
    };

    if let Some(pw) = input.password.as_deref() {
        let key = keyring::put(id, pw)?;
        let c = db.conn()?;
        repo::accounts::set_credential_key(&c, id, Some(&key))?;
    }

    let c = db.conn()?;
    Ok(repo::accounts::get(&c, id)?
        .expect("account should exist immediately after create"))
}

#[tauri::command]
pub fn update_account(
    db: State<'_, Database>,
    id: i64,
    login_url: Option<String>,
    username: Option<String>,
    requires_2fa: bool,
    notes: Option<String>,
) -> AppResult<()> {
    let c = db.conn()?;
    repo::accounts::update(
        &c,
        id,
        login_url.as_deref(),
        username.as_deref(),
        requires_2fa,
        notes.as_deref(),
    )
}

#[tauri::command]
pub fn save_account_password(
    db: State<'_, Database>,
    id: i64,
    password: String,
) -> AppResult<()> {
    let key = keyring::put(id, &password)?;
    let c = db.conn()?;
    repo::accounts::set_credential_key(&c, id, Some(&key))?;
    Ok(())
}

#[tauri::command]
pub fn clear_account_password(
    db: State<'_, Database>,
    id: i64,
) -> AppResult<()> {
    keyring::delete(id)?;
    let c = db.conn()?;
    repo::accounts::set_credential_key(&c, id, None)?;
    Ok(())
}

#[tauri::command]
pub fn reveal_account_password(id: i64) -> AppResult<Option<String>> {
    keyring::get(id)
}

#[tauri::command]
pub fn mark_account_used(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let c = db.conn()?;
    repo::accounts::mark_used(&c, id)
}

#[tauri::command]
pub fn delete_account(db: State<'_, Database>, id: i64) -> AppResult<()> {
    let _ = keyring::delete(id);
    let c = db.conn()?;
    repo::accounts::delete(&c, id)
}
