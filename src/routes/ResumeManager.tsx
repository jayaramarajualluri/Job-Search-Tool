import styles from "./Page.module.css";

export default function ResumeManager() {
  return (
    <div>
      <header className={styles.header}>
        <h1>Resume manager</h1>
      </header>
      <p className={styles.empty}>
        Canonical profile editor + per-job tailored variants. Stage 5/6.
      </p>
    </div>
  );
}
