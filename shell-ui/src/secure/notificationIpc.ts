import { runtimeRequest } from "./nativeTransport";

export interface SecureNotificationItem {
  id: string;
  title: string;
  message: string;
  severity: "info" | "warning" | "critical";
  source: string;
  createdAt: number;
  read: boolean;
  suppressed: boolean;
  actionHint?: string;
}

export interface SecureNotificationList {
  items: SecureNotificationItem[];
  unreadCount: number;
  criticalCount: number;
  quietMode: boolean;
}

function failureText(
  op: string,
  resp: { status: string; error?: { message?: string } | null },
): string {
  return `${op}: ${resp.error?.message ?? resp.status}`;
}

export async function postSecureNotification(input: {
  title: string;
  message: string;
  severity?: "info" | "warning" | "critical";
  source?: string;
  actionHint?: string;
}): Promise<boolean> {
  const resp = await runtimeRequest<Record<string, unknown>>("notification.post", {
    title: input.title,
    message: input.message,
    severity: input.severity ?? "info",
    source: input.source ?? "system",
    action_hint: input.actionHint,
  });
  if (!resp.ok) {
    throw new Error(failureText("notification.post", resp));
  }
  return resp.data?.suppressed !== true;
}

export function toastEligible(item: SecureNotificationItem): boolean {
  return !item.read && !item.suppressed;
}

export function parseNotificationList(data: Record<string, unknown>): SecureNotificationList {
  const raw = Array.isArray(data.notifications) ? data.notifications : [];
  const items: SecureNotificationItem[] = raw.map((row) => {
    const n = (row ?? {}) as Record<string, unknown>;
    const sev = n.severity === "critical" || n.severity === "warning" ? n.severity : "info";
    return {
      id: String(n.id ?? ""),
      title: String(n.title ?? ""),
      message: String(n.message ?? ""),
      severity: sev,
      source: String(n.source ?? "system"),
      createdAt: Number(n.created_at ?? 0),
      read: n.read === true,
      suppressed: n.suppressed === true,
      actionHint: typeof n.action_hint === "string" ? n.action_hint : undefined,
    };
  });
  return {
    items,
    unreadCount: Number(data.unread_count ?? 0),
    criticalCount: Number(data.critical_count ?? 0),
    quietMode: data.kiosk_quiet_mode === true,
  };
}

export async function listSecureNotifications(): Promise<SecureNotificationList> {
  const resp = await runtimeRequest<Record<string, unknown>>("notification.list", {});
  if (!resp.ok || !resp.data || typeof resp.data !== "object") {
    throw new Error(failureText("notification.list", resp));
  }
  return parseNotificationList(resp.data);
}

export async function dismissSecureNotification(
  notificationId: string,
  markRead: boolean,
): Promise<void> {
  const resp = await runtimeRequest("notification.dismiss", {
    notification_id: notificationId,
    mark_read: markRead,
    clear_all: false,
  });
  if (!resp.ok) {
    throw new Error(failureText("notification.dismiss", resp));
  }
}

export async function toggleQuietMode(): Promise<boolean> {
  const resp = await runtimeRequest<Record<string, unknown>>("tray.action", {
    action: "toggle_quiet_mode",
  });
  if (!resp.ok || !resp.data) {
    throw new Error(failureText("tray.action", resp));
  }
  return resp.data.kiosk_quiet_mode === true;
}
