import styles from "./Page.module.css";

export default function Accounts() {
  return (
    <div>
      <header className={styles.header}>
        <h1>Employer portal accounts</h1>
        <button>Add account</button>
      </header>
      <p className={styles.empty}>
        Portal logins with OS-keychain-backed passwords. Stage 5/6 wires
        the create dialog and "Open portal" action.
      </p>
    </div>
  );
}
