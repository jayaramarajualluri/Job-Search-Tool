# Job Search Tool

Local, desktop-first job search assistant for a single user on STEM OPT. Collects
jobs from public ATS sources (Greenhouse / Lever / Ashby) plus manual imports,
ranks them by recency, location, work mode, sponsorship confidence, and skill
fit, tailors a 1-page resume per job, organizes everything into a local folder
tree, and tracks employer-portal logins with OS-native secret storage.

Runs on Windows and macOS. No cloud, no telemetry, no mandatory LinkedIn
dependency.

## Status

MVP under construction. See `docs/` (coming) for per-stage design notes.

## Stack

- **Shell:** Tauri 2 (Rust host + OS webview)
- **Frontend:** React 18 + TypeScript + Vite + React Router + TanStack Query
- **Backend:** Rust (Tauri commands), SQLite via `rusqlite`, `keyring-rs` for
  OS-native secret storage, `reqwest` for HTTP, `scraper` for HTML parsing
- **Resume PDF:** HTML template → webview print-to-PDF (pluggable renderer)

## Setup & run (development)

Prereqs: Node 20+, Rust stable, platform toolchain per
<https://tauri.app/start/prerequisites/>.

```bash
npm install
npm run tauri dev
```

## Packaging

```bash
npm run tauri build        # produces .msi on Windows, .dmg on macOS
```

See `docs/PACKAGING.md` for signing notes.
