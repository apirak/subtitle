import styles from './Dropdown.module.css'

interface DropdownProps {
  value: string
  options: { value: string; label: string }[]
  onChange: (value: string) => void
}

export function Dropdown({ value, options, onChange }: DropdownProps) {
  return (
    <select
      class={styles.select}
      value={value}
      onChange={(e) => onChange((e.target as HTMLSelectElement).value)}
    >
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  )
}
