import styles from "./Page.module.css";

export default function Import() {
  return (
    <div>
      <header className={styles.header}>
        <h1>Import jobs</h1>
      </header>
      <section className={styles.card}>
        <h3>Paste a job URL</h3>
        <p className={styles.empty}>Wired in Stage 4/6.</p>
      </section>
      <section className={styles.card}>
        <h3>Paste a job description</h3>
        <p className={styles.empty}>Wired in Stage 4/6.</p>
      </section>
      <section className={styles.card}>
        <h3>LinkedIn discovery (home-only)</h3>
        <p className={styles.empty}>
          Paste a LinkedIn job URL. The app will try to resolve the official
          employer/ATS URL and use that as the source of truth. Disabled if
          LinkedIn discovery is turned off in Settings.
        </p>
      </section>
      <section className={styles.card}>
        <h3>Run scheduled ingestion</h3>
        <p className={styles.empty}>
          Queries Greenhouse / Lever / Ashby boards configured in Settings.
        </p>
      </section>
    </div>
  );
}
