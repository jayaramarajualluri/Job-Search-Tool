# Job Search Tool — Claude Code memory

Local-only Tauri 2 desktop app (Rust + React/TypeScript) that ingests public
ATS boards (Greenhouse / Lever / Ashby), ranks jobs, tailors resumes, and
tracks the application pipeline. Targets STEM-OPT users who need visa
sponsorship tracking. No cloud, no telemetry.

## Stack

| Layer | Technology |
|---|---|
| Shell | Tauri 2 (Rust host, OS webview) |
| Frontend | React 18, TypeScript, Vite, React Router, TanStack Query |
| Backend | Rust stable; `rusqlite` for SQLite; `keyring-rs` for OS secret store |
| HTTP/scraping | `reqwest`, `scraper` |
| Resume PDF | HTML template → webview print-to-PDF |

## Dev commands

```bash
npm install           # install JS deps
npm run tauri dev     # start dev server + Tauri window
npm run tauri build   # produce .msi / .dmg
```

Rust is compiled automatically by `tauri dev`. There is no separate `cargo`
step needed in development.

## Repository layout

```
src/                          React/TS frontend
  App.tsx                     Router root
  components/
    AppShell.tsx              Sidebar navigation + layout
    Badges.tsx / .module.css  All reusable badge/chip/pill widgets
    JobDetailPanel.tsx        Slide-in preview panel (opened from Dashboard)
  lib/
    ipc.ts                    ALL Tauri invoke() wrappers — one fn per command
    types.ts                  Mirror of Rust domain types (keep in sync)
  routes/
    Dashboard.tsx             Main job list with filters, stats bar, keyboard nav
    JobDetail.tsx             Full-page job detail with score, skills, files
    Companies.tsx             Company list with inline notes editing
    Accounts.tsx              Portal-login account management
    Import.tsx                Single-job URL / text import
    ImportCsv.tsx             Bulk CSV import (frontend-only parser)
    ResumeManager.tsx         Canonical profile editor
    RunHistory.tsx            Ingestion run log
    Settings.tsx              App settings editor

src-tauri/src/
  lib.rs                      Tauri app setup + command registration (invoke_handler!)
  commands/                   Thin IPC handlers — no business logic here
    jobs.rs                   list_jobs, get_job, update_job_status
    companies.rs              list_companies, update_company_notes
    accounts.rs               CRUD for portal accounts + keychain ops
    ingestion.rs              run_ingestion, import_url, import_text, resolve_linkedin_url
    resumes.rs                load/save canonical profile, list_resumes_for_job, tailor_resume_for_job
    settings.rs               load_settings, save_settings
    files.rs                  open_path, open_url
  db/repo/                    SQL repositories (one file per entity)
  domain/                     Strongly-typed business entities
  ingestion/                  ATS connectors (greenhouse, lever, ashby) + normalizer
  ranking/                    Scoring engine
  resume/                     Profile + tailoring + renderer
  files/                      Cross-platform folder organizer
  secrets/                    OS keychain wrapper
```

## Adding a new IPC command (full checklist)

1. Write the SQL/logic in `src-tauri/src/db/repo/<entity>.rs`
2. Add a `#[tauri::command]` fn in `src-tauri/src/commands/<entity>.rs`
3. Register it in the `invoke_handler!` macro inside `src-tauri/src/lib.rs`
4. Add a typed wrapper in `src/lib/ipc.ts`
5. If the command returns a new entity, add the type to `src/lib/types.ts`

## Key domain types (src/lib/types.ts)

- `Job` — central entity; has matchScore, matchExplanation, status, badges fields
- `Company` — has optional `notes` (editable via `update_company_notes`)
- `JobStatus` — new | resume_prepared | ready_to_apply | applied | oa_received | interview | rejected | closed | skipped
- `SponsorshipConfidence` — explicit_sponsor | sponsor_likely | sponsor_unclear | sponsor_unlikely | explicit_no_sponsorship
- `WorkMode` — remote | hybrid | onsite | unknown
- `RecencyBucket` — today | yesterday | week | two_weeks | month | older
- `AppSettings` — contains `targetSkills[]`, `boardSlugs`, `preferredLocations`, etc.

## Implemented features (as of this branch)

- Dashboard with stats bar (new / applied / interview / rejected counts)
- Filters: min score, recency window, status, company, work mode
- Export current filtered list to CSV
- j/k keyboard navigation; Enter = preview panel; o = open folder; a = apply URL
- JobDetailPanel slide-in: score breakdown, actions, JD summary
- JobDetail full page: skills matched/missing vs targetSkills, score bars, files, accounts
- Inline company notes editing (Companies screen)
- Bulk CSV import (/import-csv) — parses company_name, role_title, apply_url, jd_text

## ROADMAP items not yet implemented

See `docs/ROADMAP.md` for the full list. Highest-value remaining items:
- Keyboard shortcut `s` to skip a job directly from the list
- Duplicate merging in the normalizer
- Scheduled background ingestion (Tauri background task)
- Typst / Overleaf resume renderer
- SQLite snapshot export/import UI
- Linux bundle target
