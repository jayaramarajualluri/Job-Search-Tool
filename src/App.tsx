import { Routes, Route, Navigate } from "react-router-dom";
import AppShell from "./components/AppShell";
import Dashboard from "./routes/Dashboard";
import JobDetail from "./routes/JobDetail";
import Companies from "./routes/Companies";
import Accounts from "./routes/Accounts";
import ResumeManager from "./routes/ResumeManager";
import Import from "./routes/Import";
import ImportCsv from "./routes/ImportCsv";
import RunHistory from "./routes/RunHistory";
import Settings from "./routes/Settings";

export default function App() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<Dashboard />} />
        <Route path="jobs/:id" element={<JobDetail />} />
        <Route path="companies" element={<Companies />} />
        <Route path="accounts" element={<Accounts />} />
        <Route path="resumes" element={<ResumeManager />} />
        <Route path="import" element={<Import />} />
        <Route path="import-csv" element={<ImportCsv />} />
        <Route path="runs" element={<RunHistory />} />
        <Route path="settings" element={<Settings />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
