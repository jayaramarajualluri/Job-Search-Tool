import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import type { Job, JobStatus } from "@/lib/types";
import {
  LocationBadge,
  RecencyBadge,
  ScorePill,
  SponsorshipBadge,
  StatusBadge,
  WorkModeBadge,
} from "./Badges";
import styles from "./JobDetailPanel.module.css";

export default function JobDetailPanel({
  job,
  companyName,
}: {
  job: Job | null;
  companyName: (id: number) => string;
}) {
  const qc = useQueryClient();

  const setStatus = useMutation({
    mutationFn: (s: JobStatus) =>
      job ? ipc.updateJobStatus(job.id, s) : Promise.resolve(),
    onSettled: () => qc.invalidateQueries({ queryKey: ["jobs"] }),
  });

  const tailor = useMutation({
    mutationFn: () => (job ? ipc.tailorResumeForJob(job.id) : Promise.resolve("")),
    onSuccess: (path) => {
      if (path) alert(`Tailored resume written to:\n${path}`);
      qc.invalidateQueries({ queryKey: ["jobs"] });
    },
    onError: (err) => alert(`Tailor failed: ${(err as Error).message}`),
  });

  const skills = useQuery({
    queryKey: ["job-skills", job?.id],
    queryFn: () => Promise.resolve([] as string[]),
    enabled: false,
  });
  void skills;

  if (!job) {
    return (
      <aside className={styles.panel}>
        <div className={styles.empty}>
          <p>Select a job on the left to see details, score breakdown, and actions.</p>
        </div>
      </aside>
    );
  }

  const expl = job.matchExplanation;

  return (
    <aside className={styles.panel}>
      <header className={styles.header}>
        <div className={styles.titleRow}>
          <h2 className={styles.title}>{job.roleTitle}</h2>
          <ScorePill value={job.matchScore} />
        </div>
        <div className={styles.subtitle}>
          {companyName(job.companyId)}
          {job.jobExternalId && <span className={styles.dim}> · {job.jobExternalId}</span>}
        </div>
        <div className={styles.badges}>
          <WorkModeBadge value={job.workMode} />
          <LocationBadge text={job.location} />
          <RecencyBadge value={job.recencyBucket} />
          <SponsorshipBadge value={job.sponsorshipConfidence} reason={job.sponsorshipReason} />
          <StatusBadge value={job.status} />
        </div>
      </header>

      <div className={styles.actions}>
        {job.applyUrl && (
          <button onClick={() => ipc.openUrl(job.applyUrl!)}>Open job</button>
        )}
        {job.roleFolderPath && (
          <button onClick={() => ipc.openPath(job.roleFolderPath!)}>Open folder</button>
        )}
        {job.resumeFilePath ? (
          <button onClick={() => ipc.openPath(job.resumeFilePath!)}>Open resume</button>
        ) : (
          <button disabled={tailor.isPending} onClick={() => tailor.mutate()}>
            {tailor.isPending ? "Tailoring…" : "Tailor resume"}
          </button>
        )}
        <select
          value={job.status}
          onChange={(e) => setStatus.mutate(e.target.value as JobStatus)}
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

      {expl && (
        <section className={styles.section}>
          <h3>Why this matched</h3>
          <div className={styles.scoreGrid}>
            <ScoreBar label="Title" value={expl.title} />
            <ScoreBar label="Skills" value={expl.skills} />
            <ScoreBar label="Keywords" value={expl.keywords} />
            <ScoreBar label="Experience" value={expl.experience} />
            <ScoreBar label="Location" value={expl.location} />
            <ScoreBar label="Work mode" value={expl.workMode} />
            <ScoreBar label="Sponsorship" value={expl.sponsorship} />
            <ScoreBar label="Recency" value={expl.recency} />
          </div>
          {expl.notes.length > 0 && (
            <ul className={styles.notes}>
              {expl.notes.map((n, i) => (
                <li key={i}>{n}</li>
              ))}
            </ul>
          )}
        </section>
      )}

      {job.sponsorshipReason && (
        <section className={styles.section}>
          <h3>Sponsorship signal</h3>
          <p className={styles.dim}>Matched phrase: {job.sponsorshipReason}</p>
        </section>
      )}

      {job.jdSummary && (
        <section className={styles.section}>
          <h3>Summary</h3>
          <p>{job.jdSummary}</p>
        </section>
      )}

      {job.jdText && (
        <section className={styles.section}>
          <h3>Job description</h3>
          <pre className={styles.jd}>{job.jdText}</pre>
        </section>
      )}
    </aside>
  );
}

function ScoreBar({ label, value }: { label: string; value: number }) {
  const pct = Math.round(value * 100);
  return (
    <div className={styles.scoreRow}>
      <div className={styles.scoreLabel}>{label}</div>
      <div className={styles.scoreTrack}>
        <div className={styles.scoreFill} style={{ width: `${pct}%` }} />
      </div>
      <div className={styles.scoreValue}>{pct}%</div>
    </div>
  );
}
