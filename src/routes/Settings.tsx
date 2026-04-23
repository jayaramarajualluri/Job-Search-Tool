import styles from "./Page.module.css";

export default function Settings() {
  return (
    <div>
      <header className={styles.header}>
        <h1>Settings</h1>
      </header>
      <section className={styles.card}>
        <h3>General</h3>
        <p className={styles.empty}>Root folder, folder naming format.</p>
      </section>
      <section className={styles.card}>
        <h3>Search</h3>
        <p className={styles.empty}>
          Preferred roles, target skills, aliases, preferred locations, remote
          preference.
        </p>
      </section>
      <section className={styles.card}>
        <h3>Sponsorship</h3>
        <p className={styles.empty}>
          Sensitivity (strict / balanced / permissive), whether to exclude
          explicit-no-sponsorship jobs automatically.
        </p>
      </section>
      <section className={styles.card}>
        <h3>Recency</h3>
        <p className={styles.empty}>
          Stops (default 2 / 7 / 14 / 21 / 28 days), minimum good-match score,
          minimum good-match count before expanding the window.
        </p>
      </section>
      <section className={styles.card}>
        <h3>Resume</h3>
        <p className={styles.empty}>
          Canonical profile path, LaTeX source path (optional), PDF path, max
          jobs to prepare per run.
        </p>
      </section>
      <section className={styles.card}>
        <h3>Advanced</h3>
        <p className={styles.empty}>
          LinkedIn discovery on/off, export/import DB (local-only).
        </p>
      </section>
    </div>
  );
}
