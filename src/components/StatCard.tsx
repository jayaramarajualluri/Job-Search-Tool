import styles from "./StatCard.module.css";

export interface StatCardProps {
  label: string;
  value: number | string;
  hint?: string;
  tone?: "neutral" | "good" | "warn" | "bad";
}

export default function StatCard({ label, value, hint, tone = "neutral" }: StatCardProps) {
  return (
    <div className={`${styles.card} ${styles[tone]}`}>
      <div className={styles.label}>{label}</div>
      <div className={styles.value}>{value}</div>
      {hint && <div className={styles.hint}>{hint}</div>}
    </div>
  );
}
