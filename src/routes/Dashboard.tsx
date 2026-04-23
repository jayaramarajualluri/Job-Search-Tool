import styles from "./Page.module.css";

export default function Dashboard() {
  return (
    <div>
      <header className={styles.header}>
        <h1>Dashboard</h1>
        <div className={styles.actions}>
          <button>Run ingestion</button>
        </div>
      </header>
      <p className={styles.empty}>
        Jobs will appear here once ingestion has run. Stage 6 wires the
        filter bar, bucket groups, and row actions.
      </p>
    </div>
  );
}
