// REST API wrappers — one function per backend endpoint.
// In dev, Vite proxies /api → http://localhost:3000/api.
// In production, the Axum server serves both API and the built React app.

import type {
  Account,
  AccountInput,
  AppSettings,
  CanonicalProfile,
  Company,
  IngestionRun,
  Job,
  JobStatus,
  RecencyBucket,
  Resume,
} from "./types";

const BASE = "/api";

async function apiFetch<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: body !== undefined ? { "Content-Type": "application/json" } : {},
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) throw new Error(await res.text());
  if (res.status === 204) return undefined as T;
  return res.json();
}

const apiGet  = <T>(path: string)              => apiFetch<T>("GET",    path);
const apiPost = <T>(path: string, b?: unknown) => apiFetch<T>("POST",   path, b);
const apiPut  = <T>(path: string, b: unknown)  => apiFetch<T>("PUT",    path, b);
const apiPatch= <T>(path: string, b: unknown)  => apiFetch<T>("PATCH",  path, b);
const apiDel  =    (path: string)              => apiFetch<void>("DELETE", path);

export async function ping(): Promise<string> {
  return apiGet("/ping");
}

// Jobs -----------------------------------------------------------------------

export interface JobFilter {
  minScore?: number | null;
  status?: JobStatus | null;
  recency?: RecencyBucket[] | null;
  companyId?: number | null;
  includeExplicitNoSponsorship?: boolean | null;
}

export async function listJobs(f: JobFilter = {}): Promise<Job[]> {
  const p = new URLSearchParams();
  if (f.minScore != null) p.set("minScore", String(f.minScore));
  if (f.status) p.set("status", f.status);
  if (f.companyId != null) p.set("companyId", String(f.companyId));
  if (f.includeExplicitNoSponsorship != null)
    p.set("includeExplicitNoSponsorship", String(f.includeExplicitNoSponsorship));
  const qs = p.toString();
  return apiGet(`/jobs${qs ? `?${qs}` : ""}`);
}

export async function getJob(id: number): Promise<Job | null> {
  return apiGet(`/jobs/${id}`);
}

export async function updateJobStatus(
  id: number,
  status: JobStatus,
  notes?: string,
): Promise<void> {
  return apiPatch(`/jobs/${id}/status`, { status, notes: notes ?? null });
}

// Companies ------------------------------------------------------------------

export async function listCompanies(): Promise<Company[]> {
  return apiGet("/companies");
}

export async function updateCompanyNotes(id: number, notes: string | null): Promise<void> {
  return apiPatch(`/companies/${id}/notes`, { notes });
}

// Accounts -------------------------------------------------------------------

export async function listAccounts(companyId?: number): Promise<Account[]> {
  const qs = companyId != null ? `?companyId=${companyId}` : "";
  return apiGet(`/accounts${qs}`);
}

export async function createAccount(input: AccountInput): Promise<Account> {
  return apiPost("/accounts", input);
}

export async function updateAccount(
  id: number,
  loginUrl: string | null,
  username: string | null,
  requires2fa: boolean,
  notes: string | null,
): Promise<void> {
  return apiPut(`/accounts/${id}`, { loginUrl, username, requires2fa, notes });
}

export async function saveAccountPassword(id: number, password: string): Promise<void> {
  return apiPost(`/accounts/${id}/password`, { password });
}

export async function clearAccountPassword(id: number): Promise<void> {
  return apiDel(`/accounts/${id}/password`);
}

export async function revealAccountPassword(id: number): Promise<string | null> {
  return apiGet(`/accounts/${id}/password`);
}

export async function markAccountUsed(id: number): Promise<void> {
  return apiPost(`/accounts/${id}/used`);
}

export async function deleteAccount(id: number): Promise<void> {
  return apiDel(`/accounts/${id}`);
}

// Settings -------------------------------------------------------------------

export async function loadSettings(): Promise<AppSettings> {
  return apiGet("/settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return apiPut("/settings", settings);
}

// Ingestion ------------------------------------------------------------------

export type SourceSpec =
  | { kind: "greenhouse"; slug: string }
  | { kind: "lever"; slug: string }
  | { kind: "ashby"; slug: string };

export interface IngestionOutcome {
  runId: number;
  totalFetched: number;
  totalFiltered: number;
  totalPrepared: number;
}

export async function runIngestion(sources: SourceSpec[]): Promise<IngestionOutcome> {
  return apiPost("/ingestion/run", sources);
}

export async function importUrl(
  url: string,
  companyName: string,
  roleTitle: string,
  jdText: string | null,
): Promise<number> {
  return apiPost("/ingestion/import-url", { url, companyName, roleTitle, jdText });
}

export async function importText(
  companyName: string,
  roleTitle: string,
  jdText: string,
): Promise<number> {
  return apiPost("/ingestion/import-text", { companyName, roleTitle, jdText });
}

export interface LinkedinResolution {
  canonicalUrl: string;
  resolvedToAts: boolean;
  atsHost: string | null;
  titleHint: string | null;
  companyHint: string | null;
}

export async function resolveLinkedinUrl(url: string): Promise<LinkedinResolution> {
  return apiPost("/ingestion/resolve-linkedin", { url });
}

export async function listRuns(): Promise<IngestionRun[]> {
  return apiGet("/ingestion/runs");
}

// Resumes --------------------------------------------------------------------

export async function loadCanonicalProfile(): Promise<CanonicalProfile | null> {
  return apiGet("/resumes/canonical-profile");
}

export async function saveCanonicalProfile(profile: CanonicalProfile): Promise<string> {
  return apiPost("/resumes/canonical-profile", profile);
}

export async function listResumesForJob(jobId: number): Promise<Resume[]> {
  return apiGet(`/resumes/for-job/${jobId}`);
}

export async function tailorResumeForJob(jobId: number): Promise<string> {
  return apiPost(`/resumes/tailor/${jobId}`);
}

export async function tailorResumeForJobAi(jobId: number): Promise<string> {
  return apiPost(`/resumes/tailor-ai/${jobId}`);
}

// Files / URLs ---------------------------------------------------------------

export async function openPath(path: string): Promise<void> {
  window.open(`${BASE}/files/serve?path=${encodeURIComponent(path)}`, "_blank");
}

export async function openUrl(url: string): Promise<void> {
  window.open(url, "_blank", "noopener,noreferrer");
}
