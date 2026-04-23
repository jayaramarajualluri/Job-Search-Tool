import styles from "./Page.module.css";

export default function RunHistory() {
  return (
    <div>
      <header className={styles.header}>
        <h1>Run history</h1>
      </header>
      <p className={styles.empty}>
        Per-run fetched/filtered/prepared with per-source breakdown. Stage 6.
      </p>
    </div>
  );
}
