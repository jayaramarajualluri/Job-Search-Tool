# Setup

Local-only dev setup for the Job Search Tool app. Works on **Windows** and
**macOS**. No server, no cloud account needed.

## Prerequisites

1. **Node 20+** — <https://nodejs.org/>
2. **Rust stable** — <https://rustup.rs/>
3. **Tauri platform toolchain** — follow the OS-specific section on
   <https://tauri.app/start/prerequisites/>.
   - macOS: `xcode-select --install`
   - Windows: install the “Desktop development with C++” workload from Visual
     Studio Build Tools, plus WebView2 Runtime (pre-installed on Windows 11).

## Install dependencies

```bash
npm install
```

## Run in development

```bash
npm run tauri dev
```

This launches Vite on `http://localhost:1420` and opens the Tauri window
pointing at it. Hot reload works for the frontend; Rust changes require a
rebuild (Tauri handles that automatically).

## First-run checklist

1. Open **Settings** (left sidebar).
2. Set a **Root folder** — an absolute path where per-job folders and files
   will be written (default is `~/Documents/JobSearchTool`).
3. Paste your **target skills** (one per Enter). These are matched against
   JDs to drive the skill sub-score and to re-emphasize bullets when
   tailoring the resume.
4. Add **preferred roles** (e.g. `Backend Engineer`, `Data Engineer`, `ML
   Engineer`, `Platform Engineer`). Title-fit scoring uses these.
5. Add **board slugs** to ingest automatically:
   - **Greenhouse** example: `airbnb`, `stripe`, `ramp`
   - **Lever** example: `netflix`, `ahrefs`, `mux`
   - **Ashby** example: `openai`, `anthropic`, `notion`
   Find the slug by visiting the company’s career page; it usually appears
   in the URL (`boards.greenhouse.io/{slug}`, `jobs.lever.co/{slug}`,
   `jobs.ashbyhq.com/{slug}`).
6. Optional: set **canonical profile path** to a JSON file you own that
   represents your master resume. See
   `resources/sample_profile.json` for the shape.
7. Save.

## Daily flow

1. Open the app → Dashboard.
2. Click **Run ingestion**. Jobs are fetched from the configured boards,
   normalized, scored, and — for jobs above your min-score or posted
   today/yesterday — written to `rootFolder/YYYY-MM-DD/Company/Job/`.
3. For each job you care about, click **Tailor** to generate
   `tailored_resume.html` in the role folder.
4. Use the row actions: **Apply** (open the URL), **Folder** (open in
   Finder/Explorer), **Resume** (open the HTML — print to PDF from the
   browser preview if you want a PDF).
5. Mark status from the row dropdown.

## Importing one-off jobs

Go to **Import**:
- **Paste a job URL** — for Indeed/company portal/etc.
- **Paste a JD** — when you only have the text.
- **LinkedIn discovery** — paste a LI job URL; the app tries to resolve the
  official employer/ATS URL and uses that as the source of truth. Runs only
  at home (LI is blocked at work).

## Employer portal logins

Go to **Accounts** → add one per employer portal. If you enter a password
it is written to the OS secret store (**macOS Keychain** / **Windows
Credential Manager**), never to the SQLite DB or any file. Only a lookup
handle (`credential_key`) is stored in the DB.

## Data locations

- SQLite DB + logs: `<os app data dir>/Job Search Tool/jobsearch.sqlite`
  - macOS: `~/Library/Application Support/com.jobsearchtool.app/`
  - Windows: `%APPDATA%\com.jobsearchtool.app\`
- Job asset tree: `Settings → Root folder` (user-configurable).

## Backup / moving between machines

- The SQLite DB can be copied between machines.
- **Passwords do NOT move** — they live in the per-OS secret store. On the
  new machine, re-enter passwords via **Accounts** → save.
