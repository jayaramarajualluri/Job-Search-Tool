# Edge cases & how the app handles them

This file lists edge cases that have dedicated behavior — not TODOs.

## Ingestion / normalization

- **Job with no posted date.** Kept; bucketed as `null`; recency score
  contributes 0 but the job is NOT filtered out of the window (the expander
  only rejects dated jobs outside the configured max).
- **Undated + high-score job.** Always prepared (folder + files created)
  because we treat it as potentially today.
- **Same job from two sources** (e.g. LI resolved → Greenhouse AND the same
  Greenhouse board in `boardSlugs`). Deduped by
  `(company_id, job_external_id)` partial unique index; the second write
  becomes an update.
- **Same job, no external id.** Not deduped at the schema level; the UI
  shows both. Manual cleanup via **Skipped**.
- **Location "Remote (US only)" vs "Remote (worldwide)".** Detected via the
  US-remote heuristic; non-US remote scores lower but is still kept.
- **Greenhouse/Lever/Ashby JSON changes shape.** The typed deserializer
  fails loudly; the run records the per-source error in `source_summary`
  and keeps going with the other sources.
- **Indeed / LinkedIn HTML shape changes.** Manual paste flows keep working.
  Discovery resolver falls back to "resolution uncertain".
- **Company names with Unicode / diacritics.** `normalize_name` is
  NFKD-normalized; `"Über Corp"` → `"uber-corp"`.
- **Very long JDs.** Summary is first ~400 chars of non-empty lines; full
  JD is still stored.

## Filesystem

- **Windows reserved names** (`CON`, `PRN`, `COM1`…). `safe_slug` prefixes
  with `_` so they're never used as directory names.
- **Path traversal.** `safe_join` rejects `..`, absolute overrides, and any
  segment that fails sanitization. Tested.
- **Path length.** Per-segment cap at 80 chars; total path length is
  bounded by `date/company/role` so realistic paths stay under Windows's
  260-char legacy limit without long-path opt-in.
- **Filename collisions across same-company multi-role jobs.** Folder name
  is `{jobId}_{roleSlug}` if the ATS has an id; otherwise `{roleSlug}` and
  later a `jd_keyword` fallback. Collisions within one date result in the
  latest write overwriting the metadata + JD files (desired: same posting
  refreshed).

## Sponsorship

- **Unclear signal.** Jobs with no sponsorship pattern are kept as
  `sponsor_unclear` and ranked lower, never dropped.
- **Explicit no-sponsorship + a CEO email in the JD.** The CEO email is
  not a sponsorship signal; only the explicit patterns trigger
  classification.
- **Federal/defense pattern overlap.** Clearance/citizen mentions → treated
  as `explicit_no_sponsorship`. False positives are possible; `sponsorship_reason`
  surfaces the matched phrase so the user can override the status.
- **User toggles off the auto-exclude.** Jobs labelled
  `explicit_no_sponsorship` still appear in the dashboard; their composite
  score is penalized (sponsorship sub-score = 0).

## Secrets

- **Keychain entry exists but the app DB doesn't.** `reveal_account_password`
  returns the value by id; the DB is the source of truth for existence.
  A future repair routine will reconcile dangling entries.
- **User revokes keychain access.** `get` returns a platform error; the UI
  shows the error; `has_saved_password` can be refreshed manually via
  "Clear password" → "Save password".
- **Cross-device copy of the SQLite DB.** `credential_key` still resolves
  on the new machine's keychain, but the value won't be there. The UI
  shows `has_saved_password = true` but reveal returns `None`; user saves
  the password again.

## Resume tailoring

- **Profile has no bullets.** Tailoring returns an empty experience block;
  the HTML template hides the section (`{{#if experience}}`).
- **All bullets are low-relevance for the JD.** The per-role cap still
  ships the top-N original-order bullets; we never emit zero bullets
  unless the source role had none.
- **Very similar JD at the same company.** Cosine ≥ 0.85 → `Reused`;
  0.65–0.85 → `EditedReuse`; < 0.65 → `New`. Caller can inspect the
  `similarity_score` on the `resumes` row.
- **No canonical profile configured.** `tailor_resume_for_job` returns an
  `invalid` error telling the user to set the path in Settings.

## Dashboard UX

- **No jobs match the filter.** Empty-state prompts an Import or a wider
  window.
- **Board slugs empty.** Run ingestion surfaces the error message directly
  on the button.
- **LinkedIn discovery at work.** Network call will fail; the resolver
  surfaces the error without touching DB state.
