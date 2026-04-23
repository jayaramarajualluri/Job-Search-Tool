import { NavLink, Outlet } from "react-router-dom";
import styles from "./AppShell.module.css";

const nav = [
  { to: "/", label: "Dashboard", end: true },
  { to: "/companies", label: "Companies" },
  { to: "/accounts", label: "Accounts" },
  { to: "/resumes", label: "Resumes" },
  { to: "/import", label: "Import" },
  { to: "/runs", label: "Run History" },
  { to: "/settings", label: "Settings" },
];

export default function AppShell() {
  return (
    <div className={styles.shell}>
      <aside className={styles.sidebar}>
        <div className={styles.brand}>Job Search Tool</div>
        <nav>
          {nav.map((n) => (
            <NavLink
              key={n.to}
              to={n.to}
              end={n.end}
              className={({ isActive }) =>
                `${styles.link} ${isActive ? styles.active : ""}`
              }
            >
              {n.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className={styles.main}>
        <Outlet />
      </main>
    </div>
  );
}
