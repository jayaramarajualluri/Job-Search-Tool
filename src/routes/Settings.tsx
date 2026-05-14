import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import type { AppSettings } from "@/lib/types";
import page from "./Page.module.css";

function AiSection({
  settings,
  update,
}: {
  settings: AppSettings;
  update: (patch: Partial<AppSettings>) => void;
}) {
  const qc = useQueryClient();
  const hasKey = useQuery({
    queryKey: ["ai-key"],
    queryFn: () => ipc.hasAiApiKey(),
  });
  const [draftKey, setDraftKey] = useState("");
  const save = useMutation({
    mutationFn: (k: string) => ipc.setAiApiKey(k),
    onSuccess: () => {
      setDraftKey("");
      qc.invalidateQueries({ queryKey: ["ai-key"] });
    },
  });
  const clear = useMutation({
    mutationFn: () => ipc.clearAiApiKey(),
    onSettled: () => qc.invalidateQueries({ queryKey: ["ai-key"] }),
  });

  return (
    <section className={page.card}>
      <h3>AI rewriting (opt-in)</h3>
      <p className={page.empty}>
        When enabled, after the rule-based tailoring step the app sends each
        role's selected bullets to Claude with strict instructions: preserve
        every fact, never invent skills, only rewrite wording / emphasis to
        match the JD. The bullet count must match — if Claude returns
        anything else, that section silently falls back to rule-based output.
        Your API key is stored in the OS keychain, never in the SQLite DB or
        config files.
      </p>
      <label style={{ display: "block", marginBottom: 8 }}>
        <input
          type="checkbox"
          checked={settings.aiRewriteEnabled}
          onChange={(e) => update({ aiRewriteEnabled: e.target.checked })}
        />{" "}
        Enable AI rewriting
      </label>
      <label style={{ display: "block", marginBottom: 8 }}>
        Model{" "}
        <select
          value={settings.aiModel}
          onChange={(e) => update({ aiModel: e.target.value })}
        >
          <option value="claude-opus-4-7">claude-opus-4-7 (slowest, best)</option>
          <option value="claude-sonnet-4-6">claude-sonnet-4-6 (balanced)</option>
          <option value="claude-sonnet-4-5">claude-sonnet-4-5</option>
          <option value="claude-haiku-4-5-20251001">
            claude-haiku-4-5 (fastest)
          </option>
        </select>
      </label>
      <div style={{ marginTop: 6 }}>
        <strong>API key:</strong>{" "}
        <span style={{ color: hasKey.data ? "var(--good)" : "var(--text-dim)" }}>
          {hasKey.data ? "stored in keychain" : "not set"}
        </span>
      </div>
      <div style={{ marginTop: 8, display: "flex", gap: 8 }}>
        <input
          type="password"
          placeholder="sk-ant-..."
          value={draftKey}
          onChange={(e) => setDraftKey(e.target.value)}
          style={{ flex: 1, minWidth: 220 }}
        />
        <button
          disabled={!draftKey || save.isPending}
          onClick={() => save.mutate(draftKey)}
        >
          {save.isPending ? "Saving…" : "Save key"}
        </button>
        {hasKey.data && (
          <button onClick={() => clear.mutate()} disabled={clear.isPending}>
            Clear key
          </button>
        )}
      </div>
      {save.isError && (
        <p style={{ color: "var(--bad)" }}>
          {(save.error as Error).message}
        </p>
      )}
    </section>
  );
}

export default function Settings() {
  const qc = useQueryClient();
  const q = useQuery({
    queryKey: ["settings"],
    queryFn: () => ipc.loadSettings(),
  });
  const [s, setS] = useState<AppSettings | null>(null);

  useEffect(() => {
    if (q.data) setS(q.data);
  }, [q.data]);

  const save = useMutation({
    mutationFn: (next: AppSettings) => ipc.saveSettings(next),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["settings"] }),
  });

  if (!s) {
    return (
      <div>
        <header className={page.header}>
          <h1>Settings</h1>
        </header>
        <p className={page.empty}>Loading…</p>
      </div>
    );
  }

  const update = (patch: Partial<AppSettings>) => setS({ ...s, ...patch });

  return (
    <div>
      <header className={page.header}>
        <h1>Settings</h1>
        <button onClick={() => save.mutate(s)} disabled={save.isPending}>
          {save.isPending ? "Saving…" : "Save"}
        </button>
      </header>

      <section className={page.card}>
        <h3>General</h3>
        <label>
          Root folder (absolute path)
          <input
            value={s.rootFolder}
            onChange={(e) => update({ rootFolder: e.target.value })}
            style={{ width: "100%" }}
          />
        </label>
        <label style={{ display: "block", marginTop: 8 }}>
          Folder naming format
          <input
            value={s.folderNamingFormat}
            onChange={(e) => update({ folderNamingFormat: e.target.value })}
            style={{ width: "100%" }}
          />
        </label>
      </section>

      <section className={page.card}>
        <h3>Search</h3>
        <StringList
          label="Preferred roles"
          values={s.preferredRoles}
          onChange={(v) => update({ preferredRoles: v })}
        />
        <StringList
          label="Target skills"
          values={s.targetSkills}
          onChange={(v) => update({ targetSkills: v })}
        />
        <StringList
          label="Preferred locations"
          values={s.preferredLocations}
          onChange={(v) => update({ preferredLocations: v })}
        />
        <label style={{ display: "block", marginTop: 8 }}>
          Remote preference{" "}
          <select
            value={s.remotePreference}
            onChange={(e) =>
              update({ remotePreference: e.target.value as AppSettings["remotePreference"] })
            }
          >
            <option value="remote_first">Remote first</option>
            <option value="hybrid_ok">Hybrid ok</option>
            <option value="onsite_ok">Onsite ok</option>
          </select>
        </label>
      </section>

      <section className={page.card}>
        <h3>Sponsorship</h3>
        <label>
          Sensitivity{" "}
          <select
            value={s.sponsorshipSensitivity}
            onChange={(e) =>
              update({
                sponsorshipSensitivity: e.target.value as AppSettings["sponsorshipSensitivity"],
              })
            }
          >
            <option value="strict">Strict</option>
            <option value="balanced">Balanced</option>
            <option value="permissive">Permissive</option>
          </select>
        </label>
        <label style={{ display: "block", marginTop: 8 }}>
          <input
            type="checkbox"
            checked={s.excludeExplicitNoSponsorship}
            onChange={(e) =>
              update({ excludeExplicitNoSponsorship: e.target.checked })
            }
          />{" "}
          Auto-exclude explicit "no sponsorship" jobs
        </label>
      </section>

      <section className={page.card}>
        <h3>Recency</h3>
        <label>
          Min match score (0–100){" "}
          <input
            type="number"
            min={0}
            max={100}
            value={s.minMatchScore}
            onChange={(e) => update({ minMatchScore: Number(e.target.value) })}
            style={{ width: 80 }}
          />
        </label>
        <label style={{ marginLeft: 16 }}>
          Min good matches before expanding{" "}
          <input
            type="number"
            min={0}
            value={s.minGoodMatchesBeforeExpand}
            onChange={(e) =>
              update({ minGoodMatchesBeforeExpand: Number(e.target.value) })
            }
            style={{ width: 80 }}
          />
        </label>
      </section>

      <section className={page.card}>
        <h3>Board slugs (ATS ingestion)</h3>
        <p className={page.empty}>
          One slug per board. Greenhouse: the URL path under{" "}
          <code>boards-api.greenhouse.io/v1/boards/…</code>. Lever: the slug
          under <code>api.lever.co/v0/postings/…</code>. Ashby: the slug under{" "}
          <code>api.ashbyhq.com/posting-api/job-board/…</code>.
        </p>
        <StringList
          label="Greenhouse slugs"
          values={s.boardSlugs.greenhouse}
          onChange={(v) =>
            update({ boardSlugs: { ...s.boardSlugs, greenhouse: v } })
          }
        />
        <StringList
          label="Lever slugs"
          values={s.boardSlugs.lever}
          onChange={(v) =>
            update({ boardSlugs: { ...s.boardSlugs, lever: v } })
          }
        />
        <StringList
          label="Ashby slugs"
          values={s.boardSlugs.ashby}
          onChange={(v) =>
            update({ boardSlugs: { ...s.boardSlugs, ashby: v } })
          }
        />
      </section>

      <section className={page.card}>
        <h3>Resume</h3>
        <label>
          Canonical profile path (JSON){" "}
          <input
            value={s.resumeSourceInputs.canonicalProfilePath ?? ""}
            onChange={(e) =>
              update({
                resumeSourceInputs: {
                  ...s.resumeSourceInputs,
                  canonicalProfilePath: e.target.value || null,
                },
              })
            }
            style={{ width: "100%" }}
          />
        </label>
      </section>

      <AiSection settings={s} update={update} />

      <section className={page.card}>
        <h3>PDF export</h3>
        <p className={page.empty}>
          The Tailor action renders <code>tailored_resume.pdf</code> via headless
          Chrome / Edge / Brave / Chromium — whichever you already have installed.
          If none are present the app falls back to writing
          <code> tailored_resume.html </code> instead.
        </p>
      </section>

      <section className={page.card}>
        <h3>Advanced</h3>
        <label>
          <input
            type="checkbox"
            checked={s.linkedinDiscoveryEnabled}
            onChange={(e) =>
              update({ linkedinDiscoveryEnabled: e.target.checked })
            }
          />{" "}
          LinkedIn discovery enabled (home-only)
        </label>
        <div style={{ marginTop: 10, display: "flex", gap: 12, alignItems: "center" }}>
          <label style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={(s.autoIngestionIntervalMinutes ?? 0) > 0}
              onChange={(e) =>
                update({
                  autoIngestionIntervalMinutes: e.target.checked ? 60 : null,
                })
              }
            />
            Auto-run ingestion
          </label>
          {(s.autoIngestionIntervalMinutes ?? 0) > 0 && (
            <label style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
              every
              <input
                type="number"
                min={5}
                max={1440}
                value={s.autoIngestionIntervalMinutes ?? 60}
                onChange={(e) =>
                  update({
                    autoIngestionIntervalMinutes: Number(e.target.value) || null,
                  })
                }
                style={{ width: 70 }}
              />
              minutes
            </label>
          )}
        </div>
      </section>
    </div>
  );
}

function StringList({
  label,
  values,
  onChange,
}: {
  label: string;
  values: string[];
  onChange: (v: string[]) => void;
}) {
  const [draft, setDraft] = useState("");
  return (
    <div style={{ marginTop: 10 }}>
      <div style={{ fontWeight: 600, marginBottom: 4 }}>{label}</div>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
        {values.map((v, i) => (
          <span
            key={i}
            style={{
              background: "var(--panel-2)",
              border: "1px solid var(--border)",
              borderRadius: 10,
              padding: "2px 8px",
              display: "inline-flex",
              gap: 6,
              alignItems: "center",
            }}
          >
            {v}
            <button
              onClick={() => onChange(values.filter((_, j) => j !== i))}
              style={{
                padding: 0,
                background: "transparent",
                border: 0,
                color: "var(--text-dim)",
                cursor: "pointer",
              }}
            >
              ×
            </button>
          </span>
        ))}
        <input
          value={draft}
          placeholder={`add ${label.toLowerCase()}…`}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && draft.trim()) {
              onChange([...values, draft.trim()]);
              setDraft("");
            }
          }}
        />
      </div>
    </div>
  );
}
