import { useState, type ReactNode } from "react";

export function SectionCard({
  title,
  description,
  wide,
  className,
  children,
}: {
  title: string;
  description?: string;
  wide?: boolean;
  className?: string;
  children: ReactNode;
}) {
  const classes = [wide ? "secure-card secure-card-wide" : "secure-card", className]
    .filter(Boolean)
    .join(" ");
  return (
    <section className={classes}>
      <header className="ds-section-head">
        <h2>{title}</h2>
        {description && <p className="secure-detail">{description}</p>}
      </header>
      {children}
    </section>
  );
}

export function SettingRow({
  label,
  detail,
  children,
}: {
  label: string;
  detail?: string;
  children: ReactNode;
}) {
  return (
    <div className="ds-setting-row">
      <div>
        <strong>{label}</strong>
        {detail && <p>{detail}</p>}
      </div>
      <div className="ds-setting-control">{children}</div>
    </div>
  );
}

export function StatusPill({
  label,
  value,
  state,
}: {
  label?: string;
  value: string;
  state: string;
}) {
  return (
    <span className="secure-status" data-state={state}>
      {label && <strong>{label}</strong>}
      <span>{value}</span>
    </span>
  );
}

export function Toggle({
  checked,
  disabled,
  label,
  onChange,
}: {
  checked: boolean;
  disabled?: boolean;
  label: string;
  onChange: (next: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className="ds-toggle"
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span className="ds-toggle-track" data-on={checked ? "true" : "false"}>
        <span className="ds-toggle-thumb" />
      </span>
    </button>
  );
}

export function SelectField({
  label,
  value,
  disabled,
  options,
  onChange,
}: {
  label: string;
  value: string;
  disabled?: boolean;
  options: Array<{ value: string; label: string }>;
  onChange: (value: string) => void;
}) {
  return (
    <label className="ds-field">
      {label}
      <select
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

export function TextField({
  label,
  value,
  disabled,
  type = "text",
  autoComplete = "off",
  onChange,
}: {
  label: string;
  value: string;
  disabled?: boolean;
  type?: "text" | "password";
  autoComplete?: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="ds-field">
      {label}
      <input
        type={type}
        value={value}
        disabled={disabled}
        autoComplete={autoComplete}
        spellCheck={false}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  );
}

export function SignalMeter({ percent }: { percent: number }) {
  const level = Math.max(0, Math.min(4, Math.round(percent / 25)));
  return (
    <span className="ds-signal" aria-label={`${percent}%`}>
      {[1, 2, 3, 4].map((bar) => (
        <i key={bar} data-on={bar <= level ? "true" : "false"} />
      ))}
    </span>
  );
}

export function ActionBar({ children }: { children: ReactNode }) {
  return <div className="secure-actions ds-actions">{children}</div>;
}

export function DetailsSpoiler({
  summary,
  children,
}: {
  summary: string;
  children: ReactNode;
}) {
  return (
    <details className="ds-spoiler">
      <summary>{summary}</summary>
      <div className="ds-spoiler-body">{children}</div>
    </details>
  );
}

export function HelpTip({ label, text }: { label: string; text: string }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="ds-help" onPointerDown={(event) => event.stopPropagation()}>
      <button
        type="button"
        className="ds-help-trigger"
        aria-expanded={open}
        aria-label={label}
        onClick={() => setOpen((value) => !value)}
      >
        ?
      </button>
      {open ? (
        <div className="ds-help-panel" role="note">
          <p>{text}</p>
        </div>
      ) : null}
    </div>
  );
}
