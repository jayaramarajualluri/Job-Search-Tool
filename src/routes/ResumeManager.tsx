import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import type { CanonicalProfile } from "@/lib/types";
import page from "./Page.module.css";

const EMPTY_PROFILE: CanonicalProfile = {
  name: "",
  headline: null,
  contact: {
    email: null,
    phone: null,
    location: null,
    website: null,
    linkedin: null,
    github: null,
  },
  summary: null,
  skills: [],
  experience: [],
  projects: [],
  education: [],
  certifications: [],
};

export default function ResumeManager() {
  const qc = useQueryClient();
  const q = useQuery({
    queryKey: ["canonical_profile"],
    queryFn: () => ipc.loadCanonicalProfile(),
  });
  const [profile, setProfile] = useState<CanonicalProfile | null>(null);
  const [raw, setRaw] = useState("");
  const [rawError, setRawError] = useState<string | null>(null);

  useEffect(() => {
    if (q.data !== undefined) {
      const p = q.data ?? EMPTY_PROFILE;
      setProfile(p);
      setRaw(JSON.stringify(p, null, 2));
    }
  }, [q.data]);

  const save = useMutation({
    mutationFn: (next: CanonicalProfile) => ipc.saveCanonicalProfile(next),
    onSettled: () => qc.invalidateQueries({ queryKey: ["canonical_profile"] }),
  });

  const applyRaw = () => {
    try {
      const parsed = JSON.parse(raw) as CanonicalProfile;
      setProfile(parsed);
      setRawError(null);
    } catch (e) {
      setRawError((e as Error).message);
    }
  };

  if (!profile) {
    return (
      <div>
        <header className={page.header}>
          <h1>Resume manager</h1>
        </header>
        <p className={page.empty}>Loading…</p>
      </div>
    );
  }

  return (
    <div>
      <header className={page.header}>
        <h1>Resume manager</h1>
        <div style={{ display: "flex", gap: 8 }}>
          <button onClick={applyRaw}>Apply JSON</button>
          <button
            disabled={save.isPending}
            onClick={() => save.mutate(profile)}
          >
            {save.isPending ? "Saving…" : "Save profile"}
          </button>
        </div>
      </header>

      <section className={page.card}>
        <h3>Master profile (JSON)</h3>
        <p className={page.empty}>
          This is the source of truth. Tailoring only selects / reorders /
          re-emphasizes existing content — it never fabricates. Edit here or
          point Settings → Resume → canonical profile path to a file you
          maintain externally.
        </p>
        <textarea
          rows={22}
          value={raw}
          onChange={(e) => setRaw(e.target.value)}
          style={{ width: "100%", fontFamily: "monospace", fontSize: 12 }}
        />
        {rawError && (
          <p style={{ color: "var(--bad)" }}>JSON error: {rawError}</p>
        )}
        {save.data && (
          <p style={{ color: "var(--text-dim)", marginTop: 6 }}>
            Saved to {save.data}
          </p>
        )}
      </section>

      <section className={page.card}>
        <h3>Preview</h3>
        <div>
          <b>{profile.name || "(no name)"}</b>
          {profile.headline && <span> — {profile.headline}</span>}
        </div>
        <div style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 4 }}>
          {profile.experience.length} experience entries ·{" "}
          {profile.projects.length} projects · {profile.skills.length} skill
          categories · {profile.education.length} education ·{" "}
          {profile.certifications.length} certifications
        </div>
      </section>
    </div>
  );
}
