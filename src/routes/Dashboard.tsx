import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import type { Job, JobStatus, RecencyBucket, WorkMode } from "@/lib/types";
import {
  LocationBadge,
  RecencyBadge,
  ScorePill,
  SponsorshipBadge,
  StatusBadge,
  WorkModeBadge,
} from "@/components/Badges";
import page from "./Page.module.css";
import styles from "./Dashboard.module.css";

const BUCKET_ORDER: RecencyBucket[] = [
  "today",
  "yesterday",
  "week",
  "two_weeks",
  "month",
  "older",
];
const BUCKET_LABEL: Record<RecencyBucket | "unknown", string> = {
  today: "Today",
  yesterday: "Yesterday",
  week: "Last 7 days",
  two_weeks: "Last 14 days",
  month: "Last 28 days",
  older: "Older",
  unknown: "No date",
};

export default function Dashboard() {
  const qc = useQueryClient();

  const [minScore, setMinScore] = useState<number>(0);
  const [statusFilter, setStatusFilter] = useState<JobStatus | "">("");
  const [maxDays, setMaxDays] = useState<number>(7);
  const [companyFilter, setCompanyFilter] = useState<number | "">("");
  const [workModeFilter, setWorkModeFilter] = useState<WorkMode | "">("");

  const recency: RecencyBucket[] | undefined = useMemo(() => {
    if (maxDays <= 2) return ["today", "yesterday"];
    if (maxDays <= 7) return ["today", "yesterday", "week"];
    if (maxDays <= 14) return ["today", "yesterday", "week", "two_weeks"];
    if (maxDays <= 28)
      return ["today", "yesterday", "week", "two_weeks", "month"];
    return undefined; // all buckets
  }, [maxDays]);

  const allJobs = useQuery({
    queryKey: ["jobs", "all-stats"],
    queryFn: () => ipc.listJobs({ includeExplicitNoSponsorship: true }),
  });

  const stats = useMemo(() => {
    const counts = { new: 0, applied: 0, interview: 0, rejected: 0 };
    for (const j of allJobs.data ?? []) {
      if (j.status === "new") counts.new++;
      else if (j.status === "applied") counts.applied++;
      else if (j.status === "interview" || j.status === "oa_received") counts.interview++;
      else if (j.status === "rejected") counts.rejected++;
    }
    return counts;
  }, [allJobs.data]);

  const jobs = useQuery({
    queryKey: ["jobs", { minScore, statusFilter, recency, companyFilter }],
    queryFn: () =>
      ipc.listJobs({
        minScore: minScore > 0 ? minScore : null,
        status: statusFilter || null,
        recency: recency ?? null,
        companyId: companyFilter || null,
      }),
  });

  const companies = useQuery({
    queryKey: ["companies"],
    queryFn: () => ipc.listCompanies(),
  });
  const companyName = useMemo(() => {
    const m = new Map<number, string>();
    for (const c of companies.data ?? []) m.set(c.id, c.name);
    return (id: number) => m.get(id) ?? `#${id}`;
  }, [companies.data]);

  const runIngestion = useMutation({
    mutationFn: async () => {
      const settings = await ipc.loadSettings();
      const specs: ipc.SourceSpec[] = [
        ...settings.boardSlugs.greenhouse.map((slug) => ({
          kind: "greenhouse" as const,
          slug,
        })),
        ...settings.boardSlugs.lever.map((slug) => ({
          kind: "lever" as const,
          slug,
        })),
        ...settings.boardSlugs.ashby.map((slug) => ({
          kind: "ashby" as const,
          slug,
        })),
      ];
      if (specs.length === 0) {
        throw new Error(
          "No board slugs configured. Add some in Settings → Board slugs.",
        );
      }
      return ipc.runIngestion(specs);
    },
    onSettled: () => qc.invalidateQueries({ queryKey: ["jobs"] }),
  });

  const grouped = useMemo(() => {
    const g: Record<string, Job[]> = {};
    for (const j of jobs.data ?? []) {
      if (workModeFilter && j.workMode !== workModeFilter) continue;
      const key = j.recencyBucket ?? "unknown";
      (g[key] ??= []).push(j);
    }
    return g;
  }, [jobs.data, workModeFilter]);

  return (
    <div>
      <header className={page.header}>
        <h1>Dashboard</h1>
        <div className={page.actions}>
          {runIngestion.isPending && <span>Running…</span>}
          {runIngestion.isError && (
            <span className={styles.err}>
              {(runIngestion.error as Error).message}
            </span>
          )}
          {runIngestion.data && (
            <span>
              Prepared {runIngestion.data.totalPrepared} / fetched{" "}
              {runIngestion.data.totalFetched}
            </span>
          )}
          <button
            disabled={runIngestion.isPending}
            onClick={() => runIngestion.mutate()}
          >
            Run ingestion
          </button>
        </div>
      </header>

      <div className={styles.statsBar}>
        <div className={styles.statCard}>
          <span className={styles.statLabel}>New</span>
          <span className={`${styles.statValue} ${styles.neutral}`}>{stats.new}</span>
        </div>
        <div className={styles.statCard}>
          <span className={styles.statLabel}>Applied</span>
          <span className={`${styles.statValue} ${styles.good}`}>{stats.applied}</span>
        </div>
        <div className={styles.statCard}>
          <span className={styles.statLabel}>Interview / OA</span>
          <span className={`${styles.statValue} ${styles.warn}`}>{stats.interview}</span>
        </div>
        <div className={styles.statCard}>
          <span className={styles.statLabel}>Rejected</span>
          <span className={`${styles.statValue} ${styles.bad}`}>{stats.rejected}</span>
        </div>
      </div>

      <div className={styles.filters}>
        <label>
          Min score{" "}
          <input
            type="number"
            min={0}
            max={100}
            value={minScore}
            onChange={(e) => setMinScore(Number(e.target.value))}
            style={{ width: 60 }}
          />
        </label>
        <label>
          Window{" "}
          <select
            value={maxDays}
            onChange={(e) => setMaxDays(Number(e.target.value))}
          >
            <option value={2}>Today + yesterday</option>
            <option value={7}>Last 7 days</option>
            <option value={14}>Last 14 days</option>
            <option value={28}>Last 28 days</option>
            <option value={9999}>All</option>
          </select>
        </label>
        <label>
          Status{" "}
          <select
            value={statusFilter}
            onChange={(e) =>
              setStatusFilter(e.target.value as JobStatus | "")
            }
          >
            <option value="">Any</option>
            <option value="new">New</option>
            <option value="resume_prepared">Resume ready</option>
            <option value="ready_to_apply">Ready to apply</option>
            <option value="applied">Applied</option>
            <option value="skipped">Skipped</option>
          </select>
        </label>
        <label>
          Company{" "}
          <select
            value={companyFilter}
            onChange={(e) =>
              setCompanyFilter(e.target.value ? Number(e.target.value) : "")
            }
          >
            <option value="">Any</option>
            {(companies.data ?? []).map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </label>
        <label>
          Mode{" "}
          <select
            value={workModeFilter}
            onChange={(e) => setWorkModeFilter(e.target.value as WorkMode | "")}
          >
            <option value="">Any</option>
            <option value="remote">Remote</option>
            <option value="hybrid">Hybrid</option>
            <option value="onsite">On-site</option>
          </select>
        </label>
      </div>

      {jobs.isLoading && <p className={page.empty}>Loading…</p>}
      {jobs.isError && (
        <p className={styles.err}>{(jobs.error as Error).message}</p>
      )}
      {!jobs.isLoading && (jobs.data?.length ?? 0) === 0 && (
        <p className={page.empty}>
          No jobs match these filters. Try widening the window, or{" "}
          <Link to="/import">import a job</Link>.
        </p>
      )}

      {BUCKET_ORDER.map((b) => {
        const items = grouped[b];
        if (!items?.length) return null;
        return (
          <BucketGroup
            key={b}
            label={BUCKET_LABEL[b]}
            jobs={items}
            companyName={companyName}
          />
        );
      })}
      {grouped.unknown?.length ? (
        <BucketGroup
          label={BUCKET_LABEL.unknown}
          jobs={grouped.unknown}
          companyName={companyName}
        />
      ) : null}
    </div>
  );
}

function BucketGroup({
  label,
  jobs,
  companyName,
}: {
  label: string;
  jobs: Job[];
  companyName: (id: number) => string;
}) {
  return (
    <section className={styles.bucket}>
      <h2>
        {label} <span className={styles.count}>{jobs.length}</span>
      </h2>
      <div className={styles.table}>
        <div className={`${styles.row} ${styles.head}`}>
          <div>Score</div>
          <div>Role</div>
          <div>Company</div>
          <div>Mode</div>
          <div>Location</div>
          <div>Sponsorship</div>
          <div>Status</div>
          <div>Actions</div>
        </div>
        {jobs.map((j) => (
          <JobRow key={j.id} job={j} companyName={companyName} />
        ))}
      </div>
    </section>
  );
}

function JobRow({
  job,
  companyName,
}: {
  job: Job;
  companyName: (id: number) => string;
}) {
  const qc = useQueryClient();
  const mark = useMutation({
    mutationFn: (s: JobStatus) => ipc.updateJobStatus(job.id, s),
    onSettled: () => qc.invalidateQueries({ queryKey: ["jobs"] }),
  });
  const tailor = useMutation({
    mutationFn: () => ipc.tailorResumeForJob(job.id),
    onSuccess: (path) => {
      alert(`Tailored resume written to:\n${path}`);
      qc.invalidateQueries({ queryKey: ["jobs"] });
    },
    onError: (err) => {
      alert(`Tailor failed: ${(err as Error).message}`);
    },
  });

  return (
    <div className={styles.row}>
      <div>
        <ScorePill value={job.matchScore} />
      </div>
      <div>
        <Link to={`/jobs/${job.id}`}>{job.roleTitle}</Link>
        {job.jobExternalId && (
          <span className={styles.dim}> · {job.jobExternalId}</span>
        )}
      </div>
      <div>{companyName(job.companyId)}</div>
      <div>
        <WorkModeBadge value={job.workMode} />
      </div>
      <div>
        <LocationBadge text={job.location} />
      </div>
      <div>
        <SponsorshipBadge
          value={job.sponsorshipConfidence}
          reason={job.sponsorshipReason}
        />
        <RecencyBadge value={job.recencyBucket} />
      </div>
      <div>
        <StatusBadge value={job.status} />
      </div>
      <div className={styles.actionRow}>
        {job.applyUrl && (
          <button onClick={() => ipc.openUrl(job.applyUrl!)}>Apply</button>
        )}
        {job.roleFolderPath && (
          <button onClick={() => ipc.openPath(job.roleFolderPath!)}>
            Folder
          </button>
        )}
        {job.resumeFilePath ? (
          <button onClick={() => ipc.openPath(job.resumeFilePath!)}>
            Resume
          </button>
        ) : (
          <button disabled={tailor.isPending} onClick={() => tailor.mutate()}>
            Tailor
          </button>
        )}
        <select
          value={job.status}
          onChange={(e) => mark.mutate(e.target.value as JobStatus)}
        >
          <option value="new">New</option>
          <option value="resume_prepared">Resume ready</option>
          <option value="ready_to_apply">Ready to apply</option>
          <option value="applied">Applied</option>
          <option value="oa_received">OA</option>
          <option value="interview">Interview</option>
          <option value="rejected">Rejected</option>
          <option value="closed">Closed</option>
          <option value="skipped">Skipped</option>
        </select>
      </div>
    </div>
  );
}
