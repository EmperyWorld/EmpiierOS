import { useCallback, useEffect, useState } from "react";
import {
  dismissSecureNotification,
  listSecureNotifications,
  type SecureNotificationItem,
} from "../notificationIpc";
import type { LocaleStrings } from "../locale";

const POLL_MS = 18_000;

function formatNotifyTime(epochSec: number): string {
  if (!epochSec) return "";
  try {
    return new Date(epochSec * 1000).toLocaleString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      day: "numeric",
      month: "short",
    });
  } catch {
    return "";
  }
}

interface Props {
  strings: LocaleStrings;
  pollMs?: number;
  onOpenShield?: () => void;
  onItemsChange?: (items: SecureNotificationItem[], unreadCount: number) => void;
  className?: string;
}

export function SecureNotificationList({
  strings,
  pollMs = POLL_MS,
  onOpenShield,
  onItemsChange,
  className,
}: Props) {
  const [items, setItems] = useState<SecureNotificationItem[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async (): Promise<void> => {
    try {
      const list = await listSecureNotifications();
      setItems(list.items);
      setError(null);
      onItemsChange?.(list.items, list.unreadCount);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [onItemsChange]);

  useEffect(() => {
    void reload();
    const id = window.setInterval(() => void reload(), pollMs);
    return () => window.clearInterval(id);
  }, [reload, pollMs]);

  const runAction = async (action: () => Promise<void>): Promise<void> => {
    setBusy(true);
    try {
      await action();
      await reload();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const markRead = (id: string): Promise<void> =>
    runAction(() => dismissSecureNotification(id, true));

  const remove = (id: string): Promise<void> =>
    runAction(() => dismissSecureNotification(id, false));

  const errorLine = error ? (
    <p className="secure-detail" role="alert">
      {strings.taskbarNotificationsFailed} {error}
    </p>
  ) : null;

  if (items.length === 0) {
    return (
      <>
        {errorLine}
        <p className="host-app-empty">{strings.taskbarNotificationsEmpty}</p>
      </>
    );
  }

  return (
    <>
      {errorLine}
      <ul className={className ?? "secure-taskbar-notify-list secure-notification-host-list"}>
        {items.map((item) => (
          <li
            key={item.id}
            className={
              item.read
                ? "secure-taskbar-notify-item read"
                : `secure-taskbar-notify-item severity-${item.severity}`
            }
          >
            <div className="secure-taskbar-notify-head">
              <strong>{item.title}</strong>
              <span className="secure-taskbar-notify-time">{formatNotifyTime(item.createdAt)}</span>
            </div>
            <p className="secure-detail">{item.message}</p>
            <div className="secure-inline-actions">
              {item.actionHint === "shield" && onOpenShield ? (
                <button type="button" disabled={busy} onClick={onOpenShield}>
                  {strings.taskbarNotificationsOpenShield}
                </button>
              ) : null}
              {!item.read ? (
                <button type="button" disabled={busy} onClick={() => void markRead(item.id)}>
                  {strings.taskbarNotificationsMarkRead}
                </button>
              ) : null}
              <button type="button" disabled={busy} onClick={() => void remove(item.id)}>
                {strings.taskbarNotificationsClear}
              </button>
            </div>
          </li>
        ))}
      </ul>
    </>
  );
}
