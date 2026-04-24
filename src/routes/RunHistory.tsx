import { useQuery } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import page from "./Page.module.css";

export default function RunHistory() {
  const runs = useQuery({
    queryKey: ["runs"],
    queryFn: () => ipc.listRuns(),
  });

  return (
    <div>
      <header className={page.header}>
        <h1>Run history</h1>
      </header>
      {runs.isLoading && <p className={page.empty}>Loading…</p>}
      {runs.data?.length === 0 && (
        <p className={page.empty}>
          No runs yet. Kick one off from the Dashboard.
        </p>
      )}
      <div style={{ display: "grid", gap: 8 }}>
        {(runs.data ?? []).map((r) => (
          <section key={r.id} className={page.card}>
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 8,
              }}
            >
              <div>
                <b>Run #{r.id}</b>
                <span style={{ color: "var(--text-dim)", marginLeft: 8 }}>
                  {r.startedAt} → {r.endedAt ?? "…"}
                </span>
              </div>
              <div style={{ color: "var(--text-dim)" }}>
                prepared {r.totalPrepared} / filtered {r.totalFiltered} /
                fetched {r.totalFetched}
              </div>
            </div>
            {r.sourceSummary && (
              <table
                style={{
                  width: "100%",
                  marginTop: 8,
                  fontSize: 13,
                  borderCollapse: "collapse",
                }}
              >
                <thead>
                  <tr style={{ color: "var(--text-dim)", textAlign: "left" }}>
                    <th>Source</th>
                    <th>Fetched</th>
                    <th>Filtered</th>
                    <th>Prepared</th>
                    <th>Errors</th>
                  </tr>
                </thead>
                <tbody>
                  {Object.entries(r.sourceSummary).map(([name, o]) => (
                    <tr key={name} style={{ borderTop: "1px solid var(--border)" }}>
                      <td>{name}</td>
                      <td>{o.fetched}</td>
                      <td>{o.filtered}</td>
                      <td>{o.prepared}</td>
                      <td style={{ color: "var(--bad)" }}>
                        {o.errors.length > 0 ? o.errors.join("; ") : "—"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </section>
        ))}
      </div>
    </div>
  );
}
