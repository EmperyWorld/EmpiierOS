import type { ReactNode } from "react";

export type HostAppTab = {
  id: string;
  label: string;
  count?: number;
};

export type HostAppStatusChip = {
  label: string;
  state: string;
};

type Props = {
  /** Accessible name for the tab list (usually the module title). */
  ariaLabel: string;
  tabs?: HostAppTab[];
  activeTab?: string;
  onTabChange?: (id: string) => void;
  statusChips?: HostAppStatusChip[];
  /** Optional tiles under the status chips. */
  facts?: ReactNode;
  busy?: boolean;
  message?: string | null;
  error?: string | null;
  className?: string;
  children: ReactNode;
};

/**
 * Shared host-window chrome:
 * optional tabs · status chips · value tiles · one active panel.
 * Title lives on WorkspaceFrame — not here.
 */
export function HostAppShell({
  ariaLabel,
  tabs,
  activeTab,
  onTabChange,
  statusChips,
  facts,
  busy,
  message,
  error,
  className,
  children,
}: Props) {
  const rootClass = ["host-app-root", className].filter(Boolean).join(" ");
  const showTabs = Boolean(tabs && tabs.length > 1 && activeTab && onTabChange);
  return (
    <div className={rootClass}>
      {showTabs ? (
        <nav className="secure-tabs host-app-tabs" aria-label={ariaLabel}>
          {tabs!.map((item) => (
            <button
              key={item.id}
              type="button"
              className={activeTab === item.id ? "active" : undefined}
              aria-current={activeTab === item.id ? "page" : undefined}
              disabled={busy}
              onClick={() => onTabChange!(item.id)}
            >
              {item.label}
              {item.count != null ? (
                <span className="host-app-tab-count">{item.count}</span>
              ) : null}
            </button>
          ))}
        </nav>
      ) : null}

      {statusChips && statusChips.length > 0 ? (
        <div className="host-app-status-row">
          {statusChips.map((chip) => (
            <span key={`${chip.label}-${chip.state}`} className="secure-status" data-state={chip.state}>
              {chip.label}
            </span>
          ))}
          {busy ? <span className="secure-detail">…</span> : null}
        </div>
      ) : null}

      {message ? <p className="secure-message">{message}</p> : null}
      {error ? <p className="secure-error">{error}</p> : null}

      {facts ? <div className="host-app-tiles">{facts}</div> : null}

      <div className="host-app-panel">{children}</div>
    </div>
  );
}

export function HostAppFact({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="secure-fact">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
