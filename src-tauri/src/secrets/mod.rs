//! OS-native secret storage for employer portal passwords.
//!
//! Backends:
//!   - macOS:   Keychain   (`keyring` crate)
//!   - Windows: Credential Manager (`keyring` crate)
//!
//! API shape is deliberately minimal: put / get / delete / key_for.
//! Passwords NEVER touch the SQLite DB or any file.  The DB stores only a
//! `credential_key` (the lookup handle) and a `has_saved_password` flag.
//!
//! Logging discipline: we log keys but NEVER values.  Failures never quote
//! the password's length or prefix.

pub mod keyring;

pub use keyring::{delete, get, key_for, put, SERVICE};
