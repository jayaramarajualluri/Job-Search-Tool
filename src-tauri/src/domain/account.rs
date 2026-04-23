use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortalType {
    Workday,
    Greenhouse,
    Lever,
    Icims,
    Smartrecruiters,
    Ashby,
    OracleTaleo,
    Successfactors,
    Custom,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub company_id: i64,
    pub portal_type: PortalType,
    pub login_url: Option<String>,
    pub username: Option<String>,
    /// Opaque key used to look up the password in the OS keychain.  This is
    /// NOT a secret on its own — it is the *handle*, not the password.
    pub credential_key: Option<String>,
    pub has_saved_password: bool,
    pub requires_2fa: bool,
    pub notes: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for create/update.  Password, if provided, is written to the OS
/// keychain and never persisted to the DB/filesystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInput {
    pub company_id: i64,
    pub portal_type: PortalType,
    pub login_url: Option<String>,
    pub username: Option<String>,
    /// If Some, gets written to the OS keychain then immediately zeroed.
    pub password: Option<String>,
    pub requires_2fa: bool,
    pub notes: Option<String>,
}
