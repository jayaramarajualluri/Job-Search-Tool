import { useParams, Link } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import type { JobStatus } from "@/lib/types";
import {
  LocationBadge,
  RecencyBadge,
  ScorePill,
  SponsorshipBadge,
  StatusBadge,
  WorkModeBadge,
} from "@/components/Badges";
import page from "./Page.module.css";

export default function JobDetail() {
  const { id } = useParams();
  const jobId = Number(id);
  const qc = useQueryClient();

  const job = useQuery({
    queryKey: ["job", jobId],
    queryFn: () => ipc.getJob(jobId),
    enabled: !Number.isNaN(jobId),
  });

  const companies = useQuery({
    queryKey: ["companies"],
    queryFn: () => ipc.listCompanies(),
  });

  const accounts = useQuery({
    queryKey: ["accounts", job.data?.companyId],
    queryFn: () => ipc.listAccounts(job.data?.companyId),
    enabled: !!job.data?.companyId,
  });

  const resumes = useQuery({
    queryKey: ["resumes", jobId],
    queryFn: () => ipc.listResumesForJob(jobId),
    enabled: !Number.isNaN(jobId),
  });

  const setStatus = useMutation({
    mutationFn: (s: JobStatus) => ipc.updateJobStatus(jobId, s),
    onSettled: () => qc.invalidateQueries({ queryKey: ["job", jobId] }),
  });

  const tailor = useMutation({
    mutationFn: () => ipc.tailorResumeForJob(jobId),
    onSettled: () => {
      qc.invalidateQueries({ queryKey: ["job", jobId] });
      qc.invalidateQueries({ queryKey: ["resumes", jobId] });
    },
  });

  if (job.isLoading) {
    return (
      <div>
        <header className={page.header}>
          <h1>Loading…</h1>
        </header>
      </div>
    );
  }

  const j = job.data;
  if (!j) {
    return (
      <div>
        <header className={page.header}>
          <h1>Not found</h1>
          <Link to="/">Back</Link>
        </header>
      </div>
    );
  }

  const companyName =
    companies.data?.find((c) => c.id === j.companyId)?.name ??
    `Company #${j.companyId}`;
  const expl = j.matchExplanation;

  return (
    <div>
      <header className={page.header}>
        <div>
          <h1 style={{ marginBottom: 4 }}>{j.roleTitle}</h1>
          <div style={{ color: "var(--text-dim)" }}>
            {companyName}
            {j.jobExternalId && <span> · {j.jobExternalId}</span>}
          </div>
        </div>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <ScorePill value={j.matchScore} />
          <StatusBadge value={j.status} />
          <Link to="/">Back</Link>
        </div>
      </header>

      <div style={{ display: "flex", gap: 6, flexWrap: "wrap", marginBottom: 12 }}>
        <WorkModeBadge value={j.workMode} />
        <LocationBadge text={j.location} />
        <RecencyBadge value={j.recencyBucket} />
        <SponsorshipBadge
          value={j.sponsorshipConfidence}
          reason={j.sponsorshipReason}
        />
        {j.postedDate && (
          <span style={{ color: "var(--text-dim)", alignSelf: "center" }}>
            Posted {j.postedDate}
          </span>
        )}
      </div>

      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 16 }}>
        {j.applyUrl && (
          <button onClick={() => ipc.openUrl(j.applyUrl!)}>Open apply URL</button>
        )}
        {j.sourceUrl && j.sourceUrl !== j.applyUrl && (
          <button onClick={() => ipc.openUrl(j.sourceUrl!)}>Open source URL</button>
        )}
        {j.roleFolderPath && (
          <button onClick={() => ipc.openPath(j.roleFolderPath!)}>Open folder</button>
        )}
        {j.resumeFilePath ? (
          <button onClick={() => ipc.openPath(j.resumeFilePath!)}>
            Open resume
          </button>
        ) : (
          <button
            disabled={tailor.isPending || !j.roleFolderPath}
            onClick={() => tailor.mutate()}
          >
            {tailor.isPending ? "Tailoring…" : "Tailor resume"}
          </button>
        )}
        <select
          value={j.status}
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
        <section className={page.card}>
          <h3>Score breakdown</h3>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4,1fr)", gap: 8 }}>
            <ScoreRow label="Title" value={expl.title} />
            <ScoreRow label="Skills" value={expl.skills} />
            <ScoreRow label="Keywords" value={expl.keywords} />
            <ScoreRow label="Experience" value={expl.experience} />
            <ScoreRow label="Location" value={expl.location} />
            <ScoreRow label="Work mode" value={expl.workMode} />
            <ScoreRow label="Sponsorship" value={expl.sponsorship} />
            <ScoreRow label="Recency" value={expl.recency} />
          </div>
          {expl.notes.length > 0 && (
            <ul style={{ marginTop: 10, color: "var(--text-dim)" }}>
              {expl.notes.map((n, i) => (
                <li key={i}>{n}</li>
              ))}
            </ul>
          )}
        </section>
      )}

      <section className={page.card}>
        <h3>Sponsorship</h3>
        <div>
          <SponsorshipBadge
            value={j.sponsorshipConfidence}
            reason={j.sponsorshipReason}
          />
        </div>
        {j.sponsorshipReason && (
          <p style={{ color: "var(--text-dim)", marginTop: 8 }}>
            Matched phrase: {j.sponsorshipReason}
          </p>
        )}
      </section>

      <section className={page.card}>
        <h3>Files</h3>
        <ul style={{ margin: 0 }}>
          {j.jdFilePath && (
            <li>
              JD: <code>{j.jdFilePath}</code>{" "}
              <button onClick={() => ipc.openPath(j.jdFilePath!)}>Open</button>
            </li>
          )}
          {j.resumeFilePath && (
            <li>
              Resume: <code>{j.resumeFilePath}</code>{" "}
              <button onClick={() => ipc.openPath(j.resumeFilePath!)}>
                Open
              </button>
            </li>
          )}
          {j.roleFolderPath && (
            <li>
              Folder: <code>{j.roleFolderPath}</code>{" "}
              <button onClick={() => ipc.openPath(j.roleFolderPath!)}>
                Open
              </button>
            </li>
          )}
        </ul>
      </section>

      {accounts.data && accounts.data.length > 0 && (
        <section className={page.card}>
          <h3>Portal accounts</h3>
          <ul style={{ margin: 0 }}>
            {accounts.data.map((a) => (
              <li key={a.id}>
                <b>{a.portalType}</b>
                {a.username && <> · {a.username}</>}
                {a.hasSavedPassword && <> · password saved</>}
                {a.loginUrl && (
                  <>
                    {" "}
                    <button onClick={() => ipc.openUrl(a.loginUrl!)}>
                      Open portal
                    </button>
                  </>
                )}
              </li>
            ))}
          </ul>
          <p style={{ color: "var(--text-dim)", marginTop: 8 }}>
            <Link to="/accounts">Manage accounts</Link>
          </p>
        </section>
      )}

      {resumes.data && resumes.data.length > 0 && (
        <section className={page.card}>
          <h3>Resume variants</h3>
          <ul style={{ margin: 0 }}>
            {resumes.data.map((r) => (
              <li key={r.id}>
                <b>{r.resumeVariantName}</b> · {r.reuseStrategy}
                {r.similarityScore != null && (
                  <> · sim {r.similarityScore.toFixed(2)}</>
                )}
                {r.outputResumePath && (
                  <>
                    {" "}
                    <button onClick={() => ipc.openPath(r.outputResumePath!)}>
                      Open
                    </button>
                  </>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}

      {j.jdText && (
        <section className={page.card}>
          <h3>Job description</h3>
          <pre
            style={{
              whiteSpace: "pre-wrap",
              fontFamily: "inherit",
              color: "var(--text-dim)",
              margin: 0,
            }}
          >
            {j.jdText}
          </pre>
        </section>
      )}
    </div>
  );
}

function ScoreRow({ label, value }: { label: string; value: number }) {
  const pct = Math.round(value * 100);
  return (
    <div>
      <div style={{ color: "var(--text-dim)", fontSize: 11 }}>{label}</div>
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <div
          style={{
            flex: 1,
            height: 6,
            background: "var(--panel-2)",
            borderRadius: 3,
            overflow: "hidden",
          }}
        >
          <div
            style={{
              width: `${pct}%`,
              height: "100%",
              background: "var(--accent)",
            }}
          />
        </div>
        <span style={{ fontSize: 11 }}>{pct}%</span>
      </div>
    </div>
  );
}
