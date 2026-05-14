import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import type { Job, JobStatus, RecencyBucket, WorkMode } from "@/lib/types";
import {
  ScorePill,
  SponsorshipBadge,
  StatusBadge,
  WorkModeBadge,
} from "@/components/Badges";
import StatCard from "@/components/StatCard";
import { ChipGroup } from "@/components/FilterChips";
import JobDetailPanel from "@/components/JobDetailPanel";
import page from "./Page.module.css";
import styles from "./Dashboard.module.css";

type RecencyChip = "today" | "week" | "two_weeks" | "month" | "all";
type LocationChip = "remote" | "ca" | "fl" | "ny";
type ModeChip = WorkMode;

const RECENCY_OPTS: { value: RecencyChip; label: string }[] = [
  { value: "today", label: "Today" },
  { value: "week", label: "Last 7d" },
  { value: "two_weeks", label: "Last 14d" },
  { value: "month", label: "Last 28d" },
  { value: "all", label: "All" },
];
const LOCATION_OPTS: { value: LocationChip; label: string }[] = [
  { value: "remote", label: "Remote-US" },
  { value: "ca", label: "California" },
  { value: "fl", label: "Florida" },
  { value: "ny", label: "New York" },
];
const MODE_OPTS: { value: ModeChip; label: string }[] = [
  { value: "remote", label: "Remote" },
  { value: "hybrid", label: "Hybrid" },
  { value: "onsite", label: "On-site" },
];

const recencyToBuckets = (r: RecencyChip): RecencyBucket[] | null => {
  if (r === "all") return null;
  if (r === "today") return ["today", "yesterday"];
  if (r === "week") return ["today", "yesterday", "week"];
  if (r === "two_weeks") return ["today", "yesterday", "week", "two_weeks"];
  return ["today", "yesterday", "week", "two_weeks", "month"];
};

export default function Dashboard() {
  const qc = useQueryClient();

  const [recency, setRecency] = useState<RecencyChip[]>(["week"]);
  const [locations, setLocations] = useState<LocationChip[]>([]);
  const [modes, setModes] = useState<ModeChip[]>([]);
  const [minScore, setMinScore] = useState(0);
  const [statusFilter, setStatusFilter] = useState<JobStatus | "">("");
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const recencyBuckets = useMemo(
    () => recencyToBuckets(recency[0] ?? "week"),
    [recency],
  );

  const jobs = useQuery({
    queryKey: ["jobs", { minScore, statusFilter, recencyBuckets }],
    queryFn: () =>
      ipc.listJobs({
        minScore: minScore > 0 ? minScore : null,
        status: statusFilter || null,
        recency: recencyBuckets,
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

  const settings = useQuery({
    queryKey: ["settings"],
    queryFn: () => ipc.loadSettings(),
  });

  const runIngestion = useMutation({
    mutationFn: async () => {
      const s = await ipc.loadSettings();
      const specs: ipc.SourceSpec[] = [
        ...s.boardSlugs.greenhouse.map((slug) => ({ kind: "greenhouse" as const, slug })),
        ...s.boardSlugs.lever.map((slug) => ({ kind: "lever" as const, slug })),
        ...s.boardSlugs.ashby.map((slug) => ({ kind: "ashby" as const, slug })),
      ];
      if (specs.length === 0) {
        throw new Error("No board slugs configured. Settings → Board slugs.");
      }
      return ipc.runIngestion(specs);
    },
    onSettled: () => qc.invalidateQueries({ queryKey: ["jobs"] }),
  });

  const autoMin = settings.data?.autoIngestionIntervalMinutes ?? null;
  useEffect(() => {
    if (!autoMin || autoMin <= 0) return;
    const id = setInterval(() => {
      if (!runIngestion.isPending) runIngestion.mutate();
    }, autoMin * 60 * 1000);
    return () => clearInterval(id);
  }, [autoMin, runIngestion]);

  const visibleRows = useMemo(() => {
    let rows = jobs.data ?? [];
    if (modes.length > 0) rows = rows.filter((j) => modes.includes(j.workMode));
    if (locations.length > 0) {
      rows = rows.filter((j) => {
        const loc = (j.location ?? "").toLowerCase();
        const st = j.stateOrRegion ?? "";
        return locations.some((sel) => {
          if (sel === "remote") return j.workMode === "remote";
          if (sel === "ca") return st === "CA" || loc.includes("california");
          if (sel === "fl") return st === "FL" || loc.includes("florida");
          if (sel === "ny") return st === "NY" || loc.includes("new york");
          return false;
        });
      });
    }
    return rows;
  }, [jobs.data, modes, locations]);

  const stats = useMemo(() => {
    const all = jobs.data ?? [];
    const newJobs = all.filter((j) => j.status === "new").length;
    const goodMatch = all.filter((j) => (j.matchScore ?? 0) >= 70).length;
    const sponsorLikely = all.filter(
      (j) =>
        j.sponsorshipConfidence === "explicit_sponsor" ||
        j.sponsorshipConfidence === "sponsor_likely",
    ).length;
    const ready = all.filter(
      (j) => j.status === "ready_to_apply" || j.status === "resume_prepared",
    ).length;
    return { newJobs, goodMatch, sponsorLikely, ready };
  }, [jobs.data]);

  const selectedJob =
    selectedId != null ? (jobs.data ?? []).find((j) => j.id === selectedId) ?? null : null;

  return (
    <div className={styles.shell}>
      <header className={page.header}>
        <h1>Dashboard</h1>
        <div className={page.actions}>
          {autoMin ? (
            <span className={styles.autoBadge}>Auto-ingest every {autoMin}m</span>
          ) : null}
          {runIngestion.isPending && <span className={styles.dim}>Running…</span>}
          {runIngestion.data && (
            <span className={styles.dim}>
              Prepared {runIngestion.data.totalPrepared} / fetched{" "}
              {runIngestion.data.totalFetched}
            </span>
          )}
          {runIngestion.isError && (
            <span className={styles.err}>{(runIngestion.error as Error).message}</span>
          )}
          <button
            disabled={runIngestion.isPending}
            onClick={() => runIngestion.mutate()}
          >
            Run ingestion
          </button>
        </div>
      </header>

      <div className={styles.statRow}>
        <StatCard label="New jobs" value={stats.newJobs} hint="status: new" />
        <StatCard
          label="Good matches"
          value={stats.goodMatch}
          hint="score ≥ 70"
          tone="good"
        />
        <StatCard
          label="Sponsor likely"
          value={stats.sponsorLikely}
          hint="explicit or likely"
          tone="good"
        />
        <StatCard
          label="Ready to apply"
          value={stats.ready}
          hint="resume prepared / ready"
          tone="warn"
        />
      </div>

      <div className={styles.filterBar}>
        <ChipGroup options={RECENCY_OPTS} selected={recency} onChange={setRecency} />
        <ChipGroup
          options={LOCATION_OPTS}
          selected={locations}
          onChange={setLocations}
          multi
        />
        <ChipGroup options={MODE_OPTS} selected={modes} onChange={setModes} multi />
        <div className={styles.spacer} />
        <label className={styles.inlineLabel}>
          Min score
          <input
            type="number"
            min={0}
            max={100}
            value={minScore}
            onChange={(e) => setMinScore(Number(e.target.value))}
            style={{ width: 56 }}
          />
        </label>
        <label className={styles.inlineLabel}>
          Status
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as JobStatus | "")}
          >
            <option value="">Any</option>
            <option value="new">New</option>
            <option value="resume_prepared">Resume ready</option>
            <option value="ready_to_apply">Ready to apply</option>
            <option value="applied">Applied</option>
            <option value="skipped">Skipped</option>
          </select>
        </label>
      </div>

      <div className={styles.body}>
        <div className={styles.tableWrap}>
          {jobs.isLoading && <p className={page.empty}>Loading…</p>}
          {!jobs.isLoading && visibleRows.length === 0 && (
            <p className={page.empty}>
              No jobs match these filters. Run ingestion or import jobs from the Import
              tab.
            </p>
          )}
          {visibleRows.length > 0 && (
            <div className={styles.table}>
              <div className={`${styles.row} ${styles.head}`}>
                <div>Score</div>
                <div>Role</div>
                <div>Company</div>
                <div>Mode</div>
                <div>Location</div>
                <div>Sponsorship</div>
                <div>Status</div>
              </div>
              {visibleRows.map((j) => (
                <Row
                  key={j.id}
                  job={j}
                  companyName={companyName(j.companyId)}
                  selected={j.id === selectedId}
                  onClick={() => setSelectedId(j.id)}
                />
              ))}
            </div>
          )}
        </div>
        <JobDetailPanel job={selectedJob} companyName={companyName} />
      </div>
    </div>
  );
}

function Row({
  job,
  companyName,
  selected,
  onClick,
}: {
  job: Job;
  companyName: string;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <div
      className={`${styles.row} ${selected ? styles.rowSel : ""}`}
      onClick={onClick}
    >
      <div>
        <ScorePill value={job.matchScore} />
      </div>
      <div className={styles.role}>
        {job.roleTitle}
        {job.jobExternalId && (
          <span className={styles.dim}> · {job.jobExternalId}</span>
        )}
      </div>
      <div className={styles.ellipsis}>{companyName}</div>
      <div>
        <WorkModeBadge value={job.workMode} />
      </div>
      <div className={styles.ellipsis}>{job.location ?? "—"}</div>
      <div>
        <SponsorshipBadge
          value={job.sponsorshipConfidence}
          reason={job.sponsorshipReason}
        />
      </div>
      <div>
        <StatusBadge value={job.status} />
      </div>
    </div>
  );
}
