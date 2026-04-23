import styles from "./Page.module.css";

export default function Companies() {
  return (
    <div>
      <header className={styles.header}>
        <h1>Companies</h1>
      </header>
      <p className={styles.empty}>
        Company list with folder paths and job counts. Stage 6.
      </p>
    </div>
  );
}
