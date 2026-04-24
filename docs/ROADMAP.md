# Roadmap — post-MVP enhancements

These are **not** in the MVP; they're the obvious next step for each
module. Anything marked "opt-in" stays off by default to keep the app
local/quiet.

## Ingestion

- **More ATS adapters:** Workable, SmartRecruiters JSON, Workday (requires
  company-specific host handling), iCIMS.
- **Company-career-page HTML adapters** with per-site selectors stored in
  a user-editable YAML map.
- **Scheduled ingestion** (Tauri background task every N hours) — opt-in.
- **Duplicate merging** — fuzzy `(company, role, location)` within 21 days
  when no external id is present. The normalize layer already has the
  pieces; the merger is the missing step.

## Ranking

- **Per-skill weights** so the user can mark rare/important skills as
  "must" vs "nice".
- **Experience-sub-score from seniority keywords** (e.g. "Senior" +3–5
  years requirement) instead of the neutral 0.6 default.
- **Learned weights** — opt-in local logistic-regression on user's
  Applied / Skipped feedback.

## Sponsorship

- **Per-company history.** Learn from the user's accepted offers: past
  sponsors bias future unclear labels toward `sponsor_likely`.
- **H-1B LCA data cross-reference.** Optional: ingest an offline snapshot
  of public LCA filings to label companies by recent sponsorship volume.

## Resume

- **Cover-letter generator** — optional, using the same canonical profile
  + JD tailoring pass.
- **Typst renderer** to produce a native PDF without relying on webview
  print.
- **Overleaf bridge** — push the tailored resume to an Overleaf project
  via their API (opt-in; user supplies the token).
- **LLM-assisted tailoring** — opt-in, local-model-only path that rewrites
  (within honesty constraints) bullet phrasing to match JD diction.

## UX

- **Job Detail screen** with skills matched / missing, sponsorship
  reasoning panel, file links, portal account link, notes history.
- **Keyboard-first interactions** (j/k across rows, `o` to open folder,
  `enter` to open job link).
- **Inline company notes** and company-level filters on the dashboard.
- **Export to CSV** / import from CSV for portability (no secrets).

## Storage

- **SQLite snapshot export/import** with the keyring entries explicitly
  excluded (already the contract — just needs a UI).
- **Encrypted SQLite** (SQLCipher) — currently unnecessary because the DB
  holds no secrets, but an option for users who want defense in depth.

## Platform

- **Linux build** — `keyring-rs` supports Secret Service, so it's a small
  lift once the bundle target is added.
- **Auto-update** — opt-in, off by default. Local-first spirit says the
  user triggers updates manually.

## Safety

- **Browser automation for assisted-apply** — explicitly modular and
  off-by-default, with a "best-effort" caveat in the UI. Implemented via
  the `shell` plugin + a user-chosen headless tool, never bundled.
