import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as ipc from "@/lib/ipc";
import type { Account, PortalType } from "@/lib/types";
import page from "./Page.module.css";

const PORTAL_TYPES: PortalType[] = [
  "workday",
  "greenhouse",
  "lever",
  "icims",
  "smartrecruiters",
  "ashby",
  "oracle_taleo",
  "successfactors",
  "custom",
  "unknown",
];

export default function Accounts() {
  const qc = useQueryClient();
  const accounts = useQuery({
    queryKey: ["accounts", null],
    queryFn: () => ipc.listAccounts(),
  });
  const companies = useQuery({
    queryKey: ["companies"],
    queryFn: () => ipc.listCompanies(),
  });
  const [showDialog, setShowDialog] = useState(false);

  const companyName = (id: number) =>
    companies.data?.find((c) => c.id === id)?.name ?? `#${id}`;

  return (
    <div>
      <header className={page.header}>
        <h1>Employer portal accounts</h1>
        <button onClick={() => setShowDialog(true)}>Add account</button>
      </header>

      {accounts.isLoading && <p className={page.empty}>Loading…</p>}
      {accounts.data && accounts.data.length === 0 && (
        <p className={page.empty}>
          No accounts yet. Add one to track your employer-portal logins.
          Passwords are stored in the OS keychain, never in the DB.
        </p>
      )}

      {accounts.data && accounts.data.length > 0 && (
        <div style={{ display: "grid", gap: 8 }}>
          {accounts.data.map((a) => (
            <AccountRow key={a.id} account={a} companyName={companyName(a.companyId)} />
          ))}
        </div>
      )}

      {showDialog && (
        <AccountDialog
          companies={companies.data ?? []}
          onClose={() => setShowDialog(false)}
          onSaved={() => {
            qc.invalidateQueries({ queryKey: ["accounts"] });
            setShowDialog(false);
          }}
        />
      )}
    </div>
  );
}

function AccountRow({
  account,
  companyName,
}: {
  account: Account;
  companyName: string;
}) {
  const qc = useQueryClient();
  const [revealed, setRevealed] = useState<string | null>(null);
  const [newPw, setNewPw] = useState("");

  const reveal = useMutation({
    mutationFn: () => ipc.revealAccountPassword(account.id),
    onSuccess: (p) => setRevealed(p ?? "(not saved)"),
  });
  const savePw = useMutation({
    mutationFn: () => ipc.saveAccountPassword(account.id, newPw),
    onSuccess: () => {
      setNewPw("");
      qc.invalidateQueries({ queryKey: ["accounts"] });
    },
  });
  const clearPw = useMutation({
    mutationFn: () => ipc.clearAccountPassword(account.id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["accounts"] }),
  });
  const del = useMutation({
    mutationFn: () => ipc.deleteAccount(account.id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["accounts"] }),
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
          <b>{companyName}</b> · <span>{account.portalType}</span>
          {account.username && <span> · {account.username}</span>}
          {account.requires2fa && <span> · 2FA</span>}
        </div>
        <div style={{ display: "flex", gap: 6 }}>
          {account.loginUrl && (
            <button
              onClick={async () => {
                await ipc.markAccountUsed(account.id);
                await ipc.openUrl(account.loginUrl!);
              }}
            >
              Open portal
            </button>
          )}
          <button onClick={() => del.mutate()} disabled={del.isPending}>
            Delete
          </button>
        </div>
      </div>
      {account.loginUrl && (
        <div style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 4 }}>
          {account.loginUrl}
        </div>
      )}
      {account.notes && (
        <div style={{ marginTop: 6 }}>{account.notes}</div>
      )}

      <div style={{ marginTop: 10, display: "flex", gap: 8, alignItems: "center" }}>
        <span style={{ color: "var(--text-dim)", fontSize: 12 }}>
          {account.hasSavedPassword
            ? "Password saved in keychain"
            : "No password saved"}
        </span>
        {account.hasSavedPassword && (
          <>
            <button onClick={() => reveal.mutate()} disabled={reveal.isPending}>
              Reveal
            </button>
            <button onClick={() => clearPw.mutate()} disabled={clearPw.isPending}>
              Clear
            </button>
          </>
        )}
      </div>

      {revealed != null && (
        <div
          style={{
            marginTop: 8,
            padding: 8,
            background: "var(--panel-2)",
            borderRadius: 4,
            fontFamily: "monospace",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <span>{revealed}</span>
          <button onClick={() => setRevealed(null)}>Hide</button>
        </div>
      )}

      <div style={{ marginTop: 8, display: "flex", gap: 8 }}>
        <input
          type="password"
          placeholder="Set / update password"
          value={newPw}
          onChange={(e) => setNewPw(e.target.value)}
        />
        <button
          disabled={!newPw || savePw.isPending}
          onClick={() => savePw.mutate()}
        >
          {savePw.isPending ? "Saving…" : "Save to keychain"}
        </button>
      </div>
    </section>
  );
}

function AccountDialog({
  companies,
  onClose,
  onSaved,
}: {
  companies: { id: number; name: string }[];
  onClose: () => void;
  onSaved: () => void;
}) {
  const [companyId, setCompanyId] = useState<number | "">(
    companies[0]?.id ?? "",
  );
  const [portalType, setPortalType] = useState<PortalType>("workday");
  const [loginUrl, setLoginUrl] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [requires2fa, setRequires2fa] = useState(false);
  const [notes, setNotes] = useState("");

  const create = useMutation({
    mutationFn: async () => {
      if (!companyId) throw new Error("Choose a company");
      await ipc.createAccount({
        companyId: Number(companyId),
        portalType,
        loginUrl: loginUrl || null,
        username: username || null,
        password: password || null,
        requires2fa,
        notes: notes || null,
      });
    },
    onSuccess: onSaved,
  });

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 10,
      }}
      onClick={onClose}
    >
      <div
        className={page.card}
        style={{ minWidth: 480, background: "var(--bg)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3>New employer portal account</h3>
        <div style={{ display: "grid", gap: 8, marginTop: 8 }}>
          <label>
            Company{" "}
            <select
              value={companyId}
              onChange={(e) => setCompanyId(Number(e.target.value))}
            >
              {companies.length === 0 && <option value="">No companies</option>}
              {companies.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Portal type{" "}
            <select
              value={portalType}
              onChange={(e) => setPortalType(e.target.value as PortalType)}
            >
              {PORTAL_TYPES.map((p) => (
                <option key={p} value={p}>
                  {p}
                </option>
              ))}
            </select>
          </label>
          <input
            placeholder="Login URL"
            value={loginUrl}
            onChange={(e) => setLoginUrl(e.target.value)}
          />
          <input
            placeholder="Username / email"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
          />
          <input
            type="password"
            placeholder="Password (stored in OS keychain)"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          <label>
            <input
              type="checkbox"
              checked={requires2fa}
              onChange={(e) => setRequires2fa(e.target.checked)}
            />{" "}
            Requires 2FA
          </label>
          <textarea
            placeholder="Notes"
            rows={3}
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
          />
        </div>
        <div
          style={{
            marginTop: 12,
            display: "flex",
            justifyContent: "flex-end",
            gap: 8,
          }}
        >
          <button onClick={onClose}>Cancel</button>
          <button
            disabled={create.isPending || !companyId}
            onClick={() => create.mutate()}
          >
            {create.isPending ? "Saving…" : "Save"}
          </button>
        </div>
        {create.isError && (
          <p style={{ color: "var(--bad)" }}>
            {(create.error as Error).message}
          </p>
        )}
      </div>
    </div>
  );
}
