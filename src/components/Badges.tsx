import type {
  JobStatus,
  RecencyBucket,
  SponsorshipConfidence,
  WorkMode,
} from "../lib/types";
import styles from "./Badges.module.css";

export function ScorePill({ value }: { value: number | null }) {
  if (value == null) return <span className={styles.dim}>—</span>;
  const band = value >= 75 ? "good" : value >= 55 ? "warn" : "bad";
  return <span className={`${styles.pill} ${styles[band]}`}>{Math.round(value)}</span>;
}

const sponsorshipLabel: Record<SponsorshipConfidence, string> = {
  explicit_sponsor: "Sponsors",
  sponsor_likely: "Likely sponsor",
  sponsor_unclear: "Sponsor unclear",
  sponsor_unlikely: "Sponsor unlikely",
  explicit_no_sponsorship: "No sponsorship",
};
const sponsorshipTone: Record<SponsorshipConfidence, string> = {
  explicit_sponsor: "good",
  sponsor_likely: "good",
  sponsor_unclear: "neutral",
  sponsor_unlikely: "warn",
  explicit_no_sponsorship: "bad",
};

export function SponsorshipBadge({
  value,
  reason,
}: {
  value: SponsorshipConfidence;
  reason?: string | null;
}) {
  return (
    <span
      className={`${styles.chip} ${styles[sponsorshipTone[value]]}`}
      title={reason ?? undefined}
    >
      {sponsorshipLabel[value]}
    </span>
  );
}

const workModeLabel: Record<WorkMode, string> = {
  remote: "Remote",
  hybrid: "Hybrid",
  onsite: "On-site",
  unknown: "Mode ?",
};
export function WorkModeBadge({ value }: { value: WorkMode }) {
  return <span className={`${styles.chip} ${styles.neutral}`}>{workModeLabel[value]}</span>;
}

const recencyLabel: Record<RecencyBucket, string> = {
  today: "Today",
  yesterday: "Yesterday",
  week: "Last 7 days",
  two_weeks: "Last 14 days",
  month: "Last 28 days",
  older: "Older",
};
export function RecencyBadge({ value }: { value: RecencyBucket | null }) {
  if (!value) return null;
  return <span className={`${styles.chip} ${styles.neutral}`}>{recencyLabel[value]}</span>;
}

const statusLabel: Record<JobStatus, string> = {
  new: "New",
  resume_prepared: "Resume ready",
  ready_to_apply: "Ready",
  applied: "Applied",
  oa_received: "OA",
  interview: "Interview",
  rejected: "Rejected",
  closed: "Closed",
  skipped: "Skipped",
};
const statusTone: Record<JobStatus, string> = {
  new: "neutral",
  resume_prepared: "neutral",
  ready_to_apply: "warn",
  applied: "good",
  oa_received: "good",
  interview: "good",
  rejected: "bad",
  closed: "dim",
  skipped: "dim",
};
export function StatusBadge({ value }: { value: JobStatus }) {
  return (
    <span className={`${styles.chip} ${styles[statusTone[value]]}`}>
      {statusLabel[value]}
    </span>
  );
}

export function LocationBadge({ text }: { text: string | null }) {
  if (!text) return <span className={styles.dim}>—</span>;
  return <span className={`${styles.chip} ${styles.neutral}`}>{text}</span>;
}
