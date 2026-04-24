// Mirror of src-tauri/src/domain/*.rs.  Serde uses camelCase on the wire, so
// these names must match exactly.  Keep this file dumb — no runtime logic.

export type WorkMode = "remote" | "hybrid" | "onsite" | "unknown";

export type RecencyBucket =
  | "today"
  | "yesterday"
  | "week"
  | "two_weeks"
  | "month"
  | "older";

export type SponsorshipConfidence =
  | "explicit_sponsor"
  | "sponsor_likely"
  | "sponsor_unclear"
  | "sponsor_unlikely"
  | "explicit_no_sponsorship";

export type JobStatus =
  | "new"
  | "resume_prepared"
  | "ready_to_apply"
  | "applied"
  | "oa_received"
  | "interview"
  | "rejected"
  | "closed"
  | "skipped";

export type SourceName =
  | "greenhouse"
  | "lever"
  | "ashby"
  | "indeed"
  | "linkedin"
  | "manual"
  | "employer_portal"
  | "other";

export type PortalType =
  | "workday"
  | "greenhouse"
  | "lever"
  | "icims"
  | "smartrecruiters"
  | "ashby"
  | "oracle_taleo"
  | "successfactors"
  | "custom"
  | "unknown";

export type ReuseStrategy = "new" | "reused" | "edited_reuse";

export type RemotePreference = "remote_first" | "hybrid_ok" | "onsite_ok";

export type SponsorshipSensitivity = "strict" | "balanced" | "permissive";

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

export interface Company {
  id: number;
  name: string;
  normalizedName: string;
  companyFolderPath: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface MatchExplanation {
  title: number;
  skills: number;
  keywords: number;
  experience: number;
  location: number;
  workMode: number;
  sponsorship: number;
  recency: number;
  notes: string[];
}

export interface Job {
  id: number;
  companyId: number;

  sourceName: string;
  sourceUrl: string | null;
  applyUrl: string | null;

  roleTitle: string;
  normalizedRoleTitle: string;
  jobExternalId: string | null;

  location: string | null;
  stateOrRegion: string | null;
  workMode: WorkMode;

  postedDate: string | null;
  recencyBucket: RecencyBucket | null;

  jdText: string | null;
  jdSummary: string | null;

  sponsorshipConfidence: SponsorshipConfidence;
  sponsorshipReason: string | null;

  matchScore: number | null;
  matchExplanation: MatchExplanation | null;

  roleFolderPath: string | null;
  jdFilePath: string | null;
  resumeFilePath: string | null;
  coverLetterFilePath: string | null;

  status: JobStatus;
  createdAt: string;
  updatedAt: string;
}

export interface Account {
  id: number;
  companyId: number;
  portalType: PortalType;
  loginUrl: string | null;
  username: string | null;
  credentialKey: string | null;
  hasSavedPassword: boolean;
  requires2fa: boolean;
  notes: string | null;
  lastUsedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface AccountInput {
  companyId: number;
  portalType: PortalType;
  loginUrl: string | null;
  username: string | null;
  /** Plaintext password submitted from the UI.  The backend writes it to the
   *  OS keychain and never persists it to SQLite or disk. */
  password: string | null;
  requires2fa: boolean;
  notes: string | null;
}

export interface Resume {
  id: number;
  jobId: number;
  baseResumeName: string;
  resumeVariantName: string;
  sourceResumePath: string | null;
  outputResumePath: string | null;
  reuseStrategy: ReuseStrategy;
  similarityScore: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface CanonicalProfile {
  name: string;
  headline: string | null;
  contact: Contact;
  summary: string | null;
  skills: SkillGroup[];
  experience: ExperienceEntry[];
  projects: ProjectEntry[];
  education: EducationEntry[];
  certifications: CertificationEntry[];
}

export interface Contact {
  email: string | null;
  phone: string | null;
  location: string | null;
  website: string | null;
  linkedin: string | null;
  github: string | null;
}

export interface SkillGroup {
  category: string;
  skills: string[];
}

export interface ExperienceEntry {
  company: string;
  title: string;
  location: string | null;
  startDate: string;
  endDate: string | null;
  bullets: string[];
  bulletTags: string[][];
}

export interface ProjectEntry {
  name: string;
  link: string | null;
  bullets: string[];
  bulletTags: string[][];
}

export interface EducationEntry {
  institution: string;
  degree: string;
  field: string | null;
  startDate: string | null;
  endDate: string | null;
  gpa: string | null;
  highlights: string[];
}

export interface CertificationEntry {
  name: string;
  issuer: string | null;
  date: string | null;
}

export interface SkillAliasPair {
  canonical: string;
  aliases: string[];
}

export interface RecencyThresholds {
  maxDays: number;
  stops: number[];
}

export interface ResumeSourceInputs {
  canonicalProfilePath: string | null;
  latexSourcePath: string | null;
  pdfPath: string | null;
}

export interface BoardSlugs {
  greenhouse: string[];
  lever: string[];
  ashby: string[];
}

export interface AppSettings {
  rootFolder: string;
  preferredRoles: string[];
  targetSkills: string[];
  skillAliases: SkillAliasPair[];
  preferredLocations: string[];
  remotePreference: RemotePreference;
  sponsorshipSensitivity: SponsorshipSensitivity;
  excludeExplicitNoSponsorship: boolean;
  recencyThresholds: RecencyThresholds;
  minMatchScore: number;
  minGoodMatchesBeforeExpand: number;
  maxJobsToPreparePerRun: number;
  folderNamingFormat: string;
  resumeSourceInputs: ResumeSourceInputs;
  linkedinDiscoveryEnabled: boolean;
  boardSlugs: BoardSlugs;
}

export interface SourceOutcome {
  fetched: number;
  filtered: number;
  prepared: number;
  errors: string[];
}

export interface IngestionRun {
  id: number;
  startedAt: string;
  endedAt: string | null;
  sourceSummary: Record<string, SourceOutcome> | null;
  totalFetched: number;
  totalFiltered: number;
  totalPrepared: number;
  notes: string | null;
}

export interface StatusHistoryEntry {
  id: number;
  jobId: number;
  fromStatus: string | null;
  toStatus: string;
  at: string;
  notes: string | null;
}
