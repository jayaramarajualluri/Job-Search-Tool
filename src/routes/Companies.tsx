import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import page from "./Page.module.css";

export default function Companies() {
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
          <section key={c.id} className={page.card}>
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "baseline",
                gap: 8,
              }}
            >
              <div>
                <b>{c.name}</b>
                <span style={{ color: "var(--text-dim)", marginLeft: 8 }}>
                  {jobCountByCompany.get(c.id) ?? 0} jobs
                </span>
              </div>
              <div style={{ display: "flex", gap: 6 }}>
                {c.companyFolderPath && (
                  <button onClick={() => ipc.openPath(c.companyFolderPath!)}>
                    Open folder
                  </button>
                )}
                <Link to="/accounts">Accounts</Link>
              </div>
            </div>
            {c.companyFolderPath && (
              <div style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 4 }}>
                {c.companyFolderPath}
              </div>
            )}
            {c.notes && <div style={{ marginTop: 6 }}>{c.notes}</div>}
          </section>
        ))}
      </div>
    </div>
  );
}
