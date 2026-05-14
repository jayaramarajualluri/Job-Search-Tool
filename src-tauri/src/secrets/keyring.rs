//! Thin wrapper over `keyring::Entry`.  Everything keyed by
//! `jobsearch:account:{account_id}` under a fixed service name.

use crate::error::AppResult;
use keyring::Entry;

/// The service name under which all entries are stored.  Stable across OS
/// backends (shows up as the "service" attribute on Keychain items and as
/// the target prefix on Windows Credential Manager).
pub const SERVICE: &str = "jobsearchtool";

pub fn key_for(account_id: i64) -> String {
    format!("jobsearch:account:{account_id}")
}

pub fn put(account_id: i64, password: &str) -> AppResult<String> {
    let key = key_for(account_id);
    let entry = Entry::new(SERVICE, &key)?;
    entry.set_password(password)?;
    tracing::info!(key = %key, "secret stored");
    Ok(key)
}

pub fn get(account_id: i64) -> AppResult<Option<String>> {
    let key = key_for(account_id);
    let entry = Entry::new(SERVICE, &key)?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete(account_id: i64) -> AppResult<()> {
    let key = key_for(account_id);
    let entry = Entry::new(SERVICE, &key)?;
    match entry.delete_credential() {
        Ok(()) => {
            tracing::info!(key = %key, "secret deleted");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

// ---------------------------------------------------------------------------
// Named (non-account) secrets — used for things like the AI API key.
// Stored under SERVICE with a user-chosen logical name; never echoed to logs.
// ---------------------------------------------------------------------------

pub fn put_named(name: &str, secret: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, name)?;
    entry.set_password(secret)?;
    tracing::info!(name = %name, "named secret stored");
    Ok(())
}

pub fn get_named(name: &str) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE, name)?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_named(name: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, name)?;
    match entry.delete_credential() {
        Ok(()) => {
            tracing::info!(name = %name, "named secret deleted");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
