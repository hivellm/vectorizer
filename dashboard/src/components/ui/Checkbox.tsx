/**
 * Checkbox component - Custom styled checkbox
 */

interface CheckboxProps {
  id?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
}

export default function Checkbox({ id, checked, onChange, label, disabled = false }: CheckboxProps) {
  return (
    <label className="inline-flex items-center gap-2 cursor-pointer select-none" htmlFor={id}>
      <input
        id={id}
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        disabled={disabled}
        className="w-4 h-4 rounded border-neutral-300 dark:border-neutral-600 bg-neutral-50 dark:bg-neutral-800 text-blue-600 focus:ring-2 focus:ring-blue-500 focus:ring-offset-0 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      />
      {label && (
        <span className="text-sm text-neutral-700 dark:text-neutral-300 leading-none">{label}</span>
      )}
    </label>
  );
}






















