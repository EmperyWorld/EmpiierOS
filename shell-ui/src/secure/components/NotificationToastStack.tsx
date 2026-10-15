import { useCallback, useEffect, useRef, useState } from "react";
import { toastEligible, type SecureNotificationItem } from "../notificationIpc";
import type { LocaleStrings } from "../locale";

const TOAST_TTL_MS = 12_000;
const MAX_VISIBLE = 4;
const SEEN_TOAST_KEY = "empiieros.notify.toast.ids.v1";
const MAX_SEEN_TOAST = 120;

function loadSeenToastIds(): Set<string> {
  try {
    const raw = window.sessionStorage.getItem(SEEN_TOAST_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return new Set();
    return new Set(parsed.filter((id) => typeof id === "string" && id.length > 0));
  } catch {
    return new Set();
  }
}

function saveSeenToastIds(seen: Set<string>): void {
  const list = [...seen].slice(-MAX_SEEN_TOAST);
  try {
    window.sessionStorage.setItem(SEEN_TOAST_KEY, JSON.stringify(list));
  } catch {
    /* optional */
  }
}

interface ToastEntry {
  item: SecureNotificationItem;
  expiresAt: number;
}

interface Props {
  strings: LocaleStrings;
  incoming: SecureNotificationItem[];
  onOpenShield: () => void;
  onDismissToast?: (id: string) => void;
}

export function NotificationToastStack({
  strings,
  incoming,
  onOpenShield,
  onDismissToast,
}: Props) {
  const seenRef = useRef<Set<string>>(loadSeenToastIds());
  const [toasts, setToasts] = useState<ToastEntry[]>([]);

  const enqueue = useCallback((item: SecureNotificationItem) => {
    if (!item.id || seenRef.current.has(item.id)) return;
    seenRef.current.add(item.id);
    saveSeenToastIds(seenRef.current);
    setToasts((prev) => {
      const next = [...prev, { item, expiresAt: Date.now() + TOAST_TTL_MS }];
      return next.slice(-MAX_VISIBLE);
    });
  }, []);

  useEffect(() => {
    for (const item of incoming) {
      if (toastEligible(item)) {
        enqueue(item);
      }
    }
  }, [incoming, enqueue]);

  useEffect(() => {
    if (toasts.length === 0) return;
    const id = window.setInterval(() => {
      const now = Date.now();
      setToasts((prev) => prev.filter((row) => row.expiresAt > now));
    }, 800);
    return () => window.clearInterval(id);
  }, [toasts.length]);

  const dismiss = (id: string): void => {
    setToasts((prev) => prev.filter((row) => row.item.id !== id));
    onDismissToast?.(id);
  };

  if (toasts.length === 0) return null;

  return (
    <div className="secure-notification-toast-stack" aria-live="polite">
      {toasts.map(({ item }) => (
        <article
          key={item.id}
          className={`secure-notification-toast severity-${item.severity}`}
          role="status"
        >
          <div className="secure-notification-toast-head">
            <strong>{item.title}</strong>
            <button
              type="button"
              className="secure-notification-toast-close"
              aria-label={strings.taskbarNotificationsClear}
              onClick={() => dismiss(item.id)}
            >
              ×
            </button>
          </div>
          <p className="secure-detail">{item.message}</p>
          {item.actionHint === "shield" ? (
            <button
              type="button"
              className="secure-notification-toast-action"
              onClick={() => {
                dismiss(item.id);
                onOpenShield();
              }}
            >
              {strings.taskbarNotificationsOpenShield}
            </button>
          ) : null}
        </article>
      ))}
    </div>
  );
}
