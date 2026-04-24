import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import page from "./Page.module.css";

export default function Import() {
  return (
    <div>
      <header className={page.header}>
        <h1>Import jobs</h1>
      </header>
      <UrlImport />
      <TextImport />
      <LinkedInDiscovery />
    </div>
  );
}

function UrlImport() {
  const qc = useQueryClient();
  const [url, setUrl] = useState("");
  const [company, setCompany] = useState("");
  const [role, setRole] = useState("");
  const [jd, setJd] = useState("");

  const m = useMutation({
    mutationFn: () => ipc.importUrl(url, company, role, jd || null),
    onSuccess: () => {
      setUrl("");
      setCompany("");
      setRole("");
      setJd("");
      qc.invalidateQueries({ queryKey: ["jobs"] });
    },
  });

  return (
    <section className={page.card}>
      <h3>Paste a job URL</h3>
      <p className={page.empty}>
        The URL becomes the apply link; we sniff Greenhouse / Lever / Ashby /
        Indeed / LinkedIn from the host. Pasting the JD text below improves
        scoring and resume tailoring.
      </p>
      <div style={{ display: "grid", gap: 8, maxWidth: 720 }}>
        <input
          placeholder="https://…"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
          <input
            placeholder="Company"
            value={company}
            onChange={(e) => setCompany(e.target.value)}
          />
          <input
            placeholder="Role title"
            value={role}
            onChange={(e) => setRole(e.target.value)}
          />
        </div>
        <textarea
          rows={8}
          placeholder="Optional: paste the job description"
          value={jd}
          onChange={(e) => setJd(e.target.value)}
        />
        <div>
          <button
            disabled={m.isPending || !url || !company || !role}
            onClick={() => m.mutate()}
          >
            {m.isPending ? "Importing…" : "Import"}
          </button>
          {m.isError && (
            <span style={{ marginLeft: 10, color: "var(--bad)" }}>
              {(m.error as Error).message}
            </span>
          )}
        </div>
      </div>
    </section>
  );
}

function TextImport() {
  const qc = useQueryClient();
  const [company, setCompany] = useState("");
  const [role, setRole] = useState("");
  const [jd, setJd] = useState("");

  const m = useMutation({
    mutationFn: () => ipc.importText(company, role, jd),
    onSuccess: () => {
      setCompany("");
      setRole("");
      setJd("");
      qc.invalidateQueries({ queryKey: ["jobs"] });
    },
  });

  return (
    <section className={page.card}>
      <h3>Paste a job description (no URL)</h3>
      <div style={{ display: "grid", gap: 8, maxWidth: 720 }}>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
          <input
            placeholder="Company"
            value={company}
            onChange={(e) => setCompany(e.target.value)}
          />
          <input
            placeholder="Role title"
            value={role}
            onChange={(e) => setRole(e.target.value)}
          />
        </div>
        <textarea
          rows={10}
          placeholder="Paste the full JD here"
          value={jd}
          onChange={(e) => setJd(e.target.value)}
        />
        <div>
          <button
            disabled={m.isPending || !company || !role || !jd}
            onClick={() => m.mutate()}
          >
            {m.isPending ? "Importing…" : "Import"}
          </button>
        </div>
      </div>
    </section>
  );
}

function LinkedInDiscovery() {
  const [url, setUrl] = useState("");
  const [result, setResult] = useState<ipc.LinkedinResolution | null>(null);

  const m = useMutation({
    mutationFn: () => ipc.resolveLinkedinUrl(url),
    onSuccess: (r) => setResult(r),
  });

  return (
    <section className={page.card}>
      <h3>LinkedIn discovery (home-only)</h3>
      <p className={page.empty}>
        Paste a LinkedIn job URL. The app will try to resolve the official
        employer/ATS URL and use that as the source of truth. LinkedIn itself
        is never used as a core job source.
      </p>
      <div style={{ display: "grid", gap: 8, maxWidth: 720 }}>
        <input
          placeholder="https://www.linkedin.com/jobs/view/…"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
        <div>
          <button
            disabled={m.isPending || !url.includes("linkedin.com")}
            onClick={() => m.mutate()}
          >
            {m.isPending ? "Resolving…" : "Resolve"}
          </button>
          {m.isError && (
            <span style={{ marginLeft: 10, color: "var(--bad)" }}>
              {(m.error as Error).message}
            </span>
          )}
        </div>
        {result && (
          <div className={page.card} style={{ margin: 0 }}>
            <div>
              <b>Canonical URL:</b> {result.canonicalUrl}
            </div>
            <div>
              <b>Resolved to ATS:</b>{" "}
              {result.resolvedToAts ? "Yes" : "No (LinkedIn only)"}
            </div>
            {result.atsHost && (
              <div>
                <b>ATS host:</b> {result.atsHost}
              </div>
            )}
            {result.titleHint && (
              <div>
                <b>Title hint:</b> {result.titleHint}
              </div>
            )}
            {result.companyHint && (
              <div>
                <b>Company hint:</b> {result.companyHint}
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
