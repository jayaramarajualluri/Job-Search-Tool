import styles from "./FilterChips.module.css";

export interface ChipDef<V extends string> {
  value: V;
  label: string;
}

export function ChipGroup<V extends string>({
  options,
  selected,
  onChange,
  multi = false,
}: {
  options: ChipDef<V>[];
  selected: V[];
  onChange: (next: V[]) => void;
  multi?: boolean;
}) {
  const toggle = (v: V) => {
    if (multi) {
      onChange(selected.includes(v) ? selected.filter((x) => x !== v) : [...selected, v]);
    } else {
      onChange(selected.includes(v) ? [] : [v]);
    }
  };
  return (
    <div className={styles.row}>
      {options.map((o) => (
        <button
          key={o.value}
          className={`${styles.chip} ${selected.includes(o.value) ? styles.on : ""}`}
          onClick={() => toggle(o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}
