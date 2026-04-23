//! OS keychain wrapper.  Stage 5 implements the `keyring`-backed store for
//! employer-portal credentials.  We keep the API surface tiny: put/get/delete
//! keyed by `jobsearch:account:{account_id}`.
