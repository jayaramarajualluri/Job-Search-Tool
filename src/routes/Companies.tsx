import { useState } from "react";
import { Link } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import type { Company } from "@/lib/types";
import * as ipc from "@/lib/ipc";
import page from "./Page.module.css";

export default function Companies() {
  const qc = useQueryClient();

  const companies = useQuery({
    queryKey: ["companies"],
    queryFn: () => ipc.listCompanies(),
  });
  const jobs = useQuery({
    queryKey: ["jobs", "all-for-companies"],
    queryFn: () => ipc.listJobs({ includeExplicitNoSponsorship: true }),
  });

  const jobCountByCompany = new Map<number, number>();
  for (const j of jobs.data ?? []) {
    jobCountByCompany.set(j.companyId, (jobCountByCompany.get(j.companyId) ?? 0) + 1);
  }

  return (
    <div>
      <header className={page.header}>
        <h1>Companies</h1>
      </header>
      {companies.isLoading && <p className={page.empty}>Loading…</p>}
      {companies.data?.length === 0 && (
        <p className={page.empty}>
          No companies yet. Companies are created automatically when jobs are
          ingested or imported.
        </p>
      )}
      <div style={{ display: "grid", gap: 8 }}>
        {(companies.data ?? []).map((c) => (
          <CompanyCard
            key={c.id}
            company={c}
            jobCount={jobCountByCompany.get(c.id) ?? 0}
            onSaved={() => qc.invalidateQueries({ queryKey: ["companies"] })}
          />
        ))}
      </div>
    </div>
  );
}

function CompanyCard({
  company,
  jobCount,
  onSaved,
}: {
  company: Company;
  jobCount: number;
  onSaved: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(company.notes ?? "");

  const save = useMutation({
    mutationFn: () =>
      ipc.updateCompanyNotes(company.id, draft.trim() || null),
    onSuccess: () => {
      setEditing(false);
      onSaved();
    },
  });

  return (
    <section className={page.card}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "baseline",
          gap: 8,
        }}
      >
        <div>
          <b>{company.name}</b>
          <span style={{ color: "var(--text-dim)", marginLeft: 8 }}>
            {jobCount} jobs
          </span>
        </div>
        <div style={{ display: "flex", gap: 6 }}>
          {company.companyFolderPath && (
            <button onClick={() => ipc.openPath(company.companyFolderPath!)}>
              Open folder
            </button>
          )}
          <Link to="/accounts">Accounts</Link>
          <button onClick={() => { setDraft(company.notes ?? ""); setEditing(true); }}>
            {company.notes ? "Edit notes" : "Add notes"}
          </button>
        </div>
      </div>
      {company.companyFolderPath && (
        <div style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 4 }}>
          {company.companyFolderPath}
        </div>
      )}
      {editing ? (
        <div style={{ marginTop: 8, display: "flex", flexDirection: "column", gap: 6 }}>
          <textarea
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            rows={3}
            style={{ width: "100%", resize: "vertical" }}
            autoFocus
          />
          <div style={{ display: "flex", gap: 6 }}>
            <button disabled={save.isPending} onClick={() => save.mutate()}>
              Save
            </button>
            <button onClick={() => setEditing(false)}>Cancel</button>
          </div>
        </div>
      ) : (
        company.notes && (
          <div style={{ marginTop: 6, color: "var(--text-dim)" }}>{company.notes}</div>
        )
      )}
    </section>
  );
}
