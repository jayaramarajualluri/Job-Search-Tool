import { useEffect } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import {
  LocationBadge,
  RecencyBadge,
  ScorePill,
  SponsorshipBadge,
  StatusBadge,
  WorkModeBadge,
} from "./Badges";
import styles from "./JobDetailPanel.module.css";

interface Props {
  jobId: number | null;
  onClose: () => void;
}

export default function JobDetailPanel({ jobId, onClose }: Props) {
  const job = useQuery({
    queryKey: ["job", jobId],
    queryFn: () => ipc.getJob(jobId!),
    enabled: jobId != null,
  });

  const companies = useQuery({
    queryKey: ["companies"],
    queryFn: () => ipc.listCompanies(),
  });

  useEffect(() => {
    if (jobId == null) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [jobId, onClose]);

  if (jobId == null) return null;

  const j = job.data;
  const companyName =
    companies.data?.find((c) => c.id === j?.companyId)?.name ??
    (j ? `Company #${j.companyId}` : "");

  return (
    <>
      <div className={styles.backdrop} onClick={onClose} />
      <aside className={styles.panel}>
        <div className={styles.header}>
          <div>
            {job.isLoading ? (
              <span style={{ color: "var(--text-dim)" }}>Loading…</span>
            ) : j ? (
              <>
                <div className={styles.title}>{j.roleTitle}</div>
                <div className={styles.sub}>{companyName}</div>
              </>
            ) : (
              <span style={{ color: "var(--text-dim)" }}>Not found</span>
            )}
          </div>
          <button className={styles.closeBtn} onClick={onClose} title="Close (Esc)">
            ✕
          </button>
        </div>

        {j && (
          <div className={styles.body}>
            <div className={styles.badgeRow}>
              <ScorePill value={j.matchScore} />
              <StatusBadge value={j.status} />
              <WorkModeBadge value={j.workMode} />
              <RecencyBadge value={j.recencyBucket} />
            </div>
            <div className={styles.badgeRow}>
              <LocationBadge text={j.location} />
              <SponsorshipBadge
                value={j.sponsorshipConfidence}
                reason={j.sponsorshipReason}
              />
            </div>

            {j.matchExplanation && (
              <section className={styles.section}>
                <div className={styles.sectionLabel}>Score breakdown</div>
                <div className={styles.scoreGrid}>
                  {(
                    [
                      ["Title", j.matchExplanation.title],
                      ["Skills", j.matchExplanation.skills],
                      ["Keywords", j.matchExplanation.keywords],
                      ["Experience", j.matchExplanation.experience],
                      ["Location", j.matchExplanation.location],
                      ["Mode", j.matchExplanation.workMode],
                      ["Sponsorship", j.matchExplanation.sponsorship],
                      ["Recency", j.matchExplanation.recency],
                    ] as [string, number][]
                  ).map(([label, val]) => {
                    const pct = Math.round(val * 100);
                    return (
                      <div key={label} className={styles.scoreRow}>
                        <span className={styles.scoreLabel}>{label}</span>
                        <div className={styles.bar}>
                          <div
                            className={styles.barFill}
                            style={{ width: `${pct}%` }}
                          />
                        </div>
                        <span className={styles.scorePct}>{pct}%</span>
                      </div>
                    );
                  })}
                </div>
              </section>
            )}

            <section className={styles.section}>
              <div className={styles.sectionLabel}>Actions</div>
              <div className={styles.actionRow}>
                {j.applyUrl && (
                  <button onClick={() => ipc.openUrl(j.applyUrl!)}>
                    Apply URL
                  </button>
                )}
                {j.roleFolderPath && (
                  <button onClick={() => ipc.openPath(j.roleFolderPath!)}>
                    Folder
                  </button>
                )}
                {j.resumeFilePath && (
                  <button onClick={() => ipc.openPath(j.resumeFilePath!)}>
                    Resume
                  </button>
                )}
                <Link to={`/jobs/${j.id}`} onClick={onClose}>
                  Full detail →
                </Link>
              </div>
            </section>

            {j.jdSummary && (
              <section className={styles.section}>
                <div className={styles.sectionLabel}>Summary</div>
                <p style={{ color: "var(--text-dim)", margin: 0, fontSize: 13 }}>
                  {j.jdSummary}
                </p>
              </section>
            )}
          </div>
        )}
      </aside>
    </>
  );
}
