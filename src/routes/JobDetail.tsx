import { useParams, Link } from "react-router-dom";
import styles from "./Page.module.css";

export default function JobDetail() {
  const { id } = useParams();
  return (
    <div>
      <header className={styles.header}>
        <h1>Job #{id}</h1>
        <Link to="/">Back to dashboard</Link>
      </header>
      <p className={styles.empty}>Detail panels render here in Stage 6.</p>
    </div>
  );
}
