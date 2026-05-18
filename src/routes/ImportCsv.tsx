import { useRef, useState } from "react";
import * as ipc from "@/lib/ipc";
import page from "./Page.module.css";

interface CsvRow {
  company_name: string;
  role_title: string;
  apply_url: string;
  jd_text: string;
}

function parseCsv(text: string): CsvRow[] {
  const lines = text.split(/\r?\n/).filter((l) => l.trim());
  if (lines.length < 2) return [];
  const headers = splitCsvLine(lines[0]).map((h) => h.toLowerCase().trim());
  return lines.slice(1).map((line) => {
    const vals = splitCsvLine(line);
    const get = (k: string) => vals[headers.indexOf(k)]?.trim() ?? "";
    return {
      company_name: get("company_name") || get("company"),
      role_title: get("role_title") || get("role") || get("title"),
      apply_url: get("apply_url") || get("url"),
      jd_text: get("jd_text") || get("description") || get("jd"),
    };
  });
}

function splitCsvLine(line: string): string[] {
  const result: string[] = [];
  let cur = "";
  let inQuote = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      if (inQuote && line[i + 1] === '"') { cur += '"'; i++; }
      else inQuote = !inQuote;
    } else if (ch === "," && !inQuote) {
      result.push(cur); cur = "";
    } else {
      cur += ch;
    }
  }
  result.push(cur);
  return result;
}

export default function ImportCsv() {
  const fileRef = useRef<HTMLInputElement>(null);
  const [rows, setRows] = useState<CsvRow[]>([]);
  const [errors, setErrors] = useState<string[]>([]);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [done, setDone] = useState(false);

  function handleFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      const text = ev.target?.result as string;
      const parsed = parseCsv(text);
      setRows(parsed);
      setErrors([]);
      setProgress(null);
      setDone(false);
    };
    reader.readAsText(file);
  }

  async function runImport() {
    const errs: string[] = [];
    setProgress({ done: 0, total: rows.length });
    setErrors([]);
    setDone(false);
    for (let i = 0; i < rows.length; i++) {
      const r = rows[i];
      try {
        if (r.apply_url) {
          await ipc.importUrl(r.apply_url, r.company_name, r.role_title, r.jd_text || null);
        } else {
          await ipc.importText(r.company_name, r.role_title, r.jd_text);
        }
      } catch (err) {
        errs.push(`Row ${i + 1} (${r.role_title}): ${(err as Error).message}`);
      }
      setProgress({ done: i + 1, total: rows.length });
    }
    setErrors(errs);
    setDone(true);
  }

  const validRows = rows.filter((r) => r.company_name && r.role_title);

  return (
    <div>
      <header className={page.header}>
        <h1>Import from CSV</h1>
      </header>

      <section className={page.card}>
        <h3>Expected columns</h3>
        <p style={{ color: "var(--text-dim)", margin: "4px 0 0" }}>
          <code>company_name</code>, <code>role_title</code> — required.<br />
          <code>apply_url</code> — if present, uses URL import path.<br />
          <code>jd_text</code> — full job description text (optional).<br />
          First row must be a header row. Values may be quoted.
        </p>
      </section>

      <section className={page.card}>
        <h3>Select CSV file</h3>
        <input ref={fileRef} type="file" accept=".csv,text/csv" onChange={handleFile} />
      </section>

      {rows.length > 0 && (
        <section className={page.card}>
          <h3>Preview — {rows.length} rows ({validRows.length} valid)</h3>
          <div style={{ overflowX: "auto" }}>
            <table style={{ borderCollapse: "collapse", width: "100%", fontSize: 13 }}>
              <thead>
                <tr style={{ color: "var(--text-dim)", textAlign: "left" }}>
                  <th style={{ padding: "4px 10px 4px 0" }}>#</th>
                  <th style={{ padding: "4px 10px 4px 0" }}>Company</th>
                  <th style={{ padding: "4px 10px 4px 0" }}>Role</th>
                  <th style={{ padding: "4px 10px 4px 0" }}>URL</th>
                  <th style={{ padding: "4px 10px 4px 0" }}>JD</th>
                </tr>
              </thead>
              <tbody>
                {rows.slice(0, 20).map((r, i) => {
                  const valid = !!(r.company_name && r.role_title);
                  return (
                    <tr key={i} style={{ color: valid ? "var(--text)" : "var(--bad)" }}>
                      <td style={{ padding: "3px 10px 3px 0" }}>{i + 1}</td>
                      <td style={{ padding: "3px 10px 3px 0" }}>{r.company_name || "—"}</td>
                      <td style={{ padding: "3px 10px 3px 0" }}>{r.role_title || "—"}</td>
                      <td style={{ padding: "3px 10px 3px 0" }}>
                        {r.apply_url ? "✓" : "—"}
                      </td>
                      <td style={{ padding: "3px 10px 3px 0" }}>
                        {r.jd_text ? `${r.jd_text.slice(0, 40)}…` : "—"}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
            {rows.length > 20 && (
              <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 4 }}>
                … and {rows.length - 20} more rows
              </p>
            )}
          </div>

          {!done && (
            <div style={{ marginTop: 12 }}>
              {progress ? (
                <span style={{ color: "var(--text-dim)" }}>
                  Importing {progress.done} / {progress.total}…
                </span>
              ) : (
                <button
                  disabled={validRows.length === 0}
                  onClick={runImport}
                >
                  Import {validRows.length} jobs
                </button>
              )}
            </div>
          )}

          {done && (
            <div style={{ marginTop: 12 }}>
              <span style={{ color: "var(--good)" }}>
                Done — imported {validRows.length - errors.length} / {validRows.length} jobs.
              </span>
              {errors.length > 0 && (
                <ul style={{ color: "var(--bad)", marginTop: 8, paddingLeft: 20 }}>
                  {errors.map((e, i) => <li key={i}>{e}</li>)}
                </ul>
              )}
            </div>
          )}
        </section>
      )}
    </div>
  );
}
