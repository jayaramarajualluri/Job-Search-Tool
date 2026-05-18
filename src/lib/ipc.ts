// Typed wrappers over Tauri `invoke`.  One helper per command.  Keep this
// file pure — no UI, no runtime logic beyond calling invoke().

import { invoke } from "@tauri-apps/api/core";
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

export async function ping(): Promise<string> {
  return invoke<string>("ping");
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
  return invoke<Job[]>("list_jobs", {
    minScore: f.minScore ?? null,
    status: f.status ?? null,
    recency: f.recency ?? null,
    companyId: f.companyId ?? null,
    includeExplicitNoSponsorship: f.includeExplicitNoSponsorship ?? null,
  });
}

export async function getJob(id: number): Promise<Job | null> {
  return invoke<Job | null>("get_job", { id });
}

export async function updateJobStatus(
  id: number,
  status: JobStatus,
  notes?: string,
): Promise<void> {
  return invoke("update_job_status", { id, status, notes: notes ?? null });
}

// Companies ------------------------------------------------------------------

export async function listCompanies(): Promise<Company[]> {
  return invoke<Company[]>("list_companies");
}

export async function updateCompanyNotes(
  id: number,
  notes: string | null,
): Promise<void> {
  return invoke("update_company_notes", { id, notes });
}

// Accounts -------------------------------------------------------------------

export async function listAccounts(companyId?: number): Promise<Account[]> {
  return invoke<Account[]>("list_accounts", { companyId: companyId ?? null });
}

export async function createAccount(input: AccountInput): Promise<Account> {
  return invoke<Account>("create_account", { input });
}

export async function updateAccount(
  id: number,
  loginUrl: string | null,
  username: string | null,
  requires2fa: boolean,
  notes: string | null,
): Promise<void> {
  return invoke("update_account", {
    id,
    loginUrl,
    username,
    requires2fa,
    notes,
  });
}

export async function saveAccountPassword(id: number, password: string): Promise<void> {
  return invoke("save_account_password", { id, password });
}

export async function clearAccountPassword(id: number): Promise<void> {
  return invoke("clear_account_password", { id });
}

export async function revealAccountPassword(id: number): Promise<string | null> {
  return invoke<string | null>("reveal_account_password", { id });
}

export async function markAccountUsed(id: number): Promise<void> {
  return invoke("mark_account_used", { id });
}

export async function deleteAccount(id: number): Promise<void> {
  return invoke("delete_account", { id });
}

// Settings -------------------------------------------------------------------

export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("load_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
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
  return invoke<IngestionOutcome>("run_ingestion", { sources });
}

export async function importUrl(
  url: string,
  companyName: string,
  roleTitle: string,
  jdText: string | null,
): Promise<number> {
  return invoke<number>("import_url", {
    url,
    companyName,
    roleTitle,
    jdText,
  });
}

export async function importText(
  companyName: string,
  roleTitle: string,
  jdText: string,
): Promise<number> {
  return invoke<number>("import_text", { companyName, roleTitle, jdText });
}

export interface LinkedinResolution {
  canonicalUrl: string;
  resolvedToAts: boolean;
  atsHost: string | null;
  titleHint: string | null;
  companyHint: string | null;
}

export async function resolveLinkedinUrl(url: string): Promise<LinkedinResolution> {
  return invoke<LinkedinResolution>("resolve_linkedin_url", { url });
}

export async function listRuns(): Promise<IngestionRun[]> {
  return invoke<IngestionRun[]>("list_runs");
}

// Resumes --------------------------------------------------------------------

export async function loadCanonicalProfile(): Promise<CanonicalProfile | null> {
  return invoke<CanonicalProfile | null>("load_canonical_profile");
}

export async function saveCanonicalProfile(
  profile: CanonicalProfile,
): Promise<string> {
  return invoke<string>("save_canonical_profile", { profile });
}

export async function listResumesForJob(jobId: number): Promise<Resume[]> {
  return invoke<Resume[]>("list_resumes_for_job", { jobId });
}

export async function tailorResumeForJob(jobId: number): Promise<string> {
  return invoke<string>("tailor_resume_for_job", { jobId });
}

// Files / URLs ---------------------------------------------------------------

export async function openPath(path: string): Promise<void> {
  return invoke("open_path", { path });
}

export async function openUrl(url: string): Promise<void> {
  return invoke("open_url", { url });
}
