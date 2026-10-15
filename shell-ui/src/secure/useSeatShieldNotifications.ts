import { useCallback, useEffect, useRef } from "react";
import type { LocaleStrings } from "./locale";
import { nativeBridgeAvailable } from "./nativeTransport";
import {
  listSecureNotifications,
  postSecureNotification,
  type SecureNotificationItem,
} from "./notificationIpc";
import { mapSeatShieldEvents, type SeatShieldEventView } from "../session/seatShield";
import { runtimeRequest } from "./nativeTransport";

const POLL_MS = 18_000;
const SEEN_KEY = "empiieros.seat-shield.notify.sequences.v1";
const MAX_SEEN = 400;

function loadSeen(): Set<number> {
  try {
    const raw = window.sessionStorage.getItem(SEEN_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return new Set();
    return new Set(parsed.filter((n) => typeof n === "number"));
  } catch {
    return new Set();
  }
}

function saveSeen(seen: Set<number>): void {
  const list = [...seen].slice(-MAX_SEEN);
  try {
    window.sessionStorage.setItem(SEEN_KEY, JSON.stringify(list));
  } catch {
    /* optional */
  }
}

function severityForEvent(event: SeatShieldEventView): "info" | "warning" | "critical" {
  if (event.severity === "critical") return "critical";
  if (event.severity === "high") return "warning";
  return "info";
}

function titleForEvent(strings: LocaleStrings, event: SeatShieldEventView): string {
  if (event.severity === "critical" || event.severity === "high") {
    return strings.seatShieldNotifyTitleThreat;
  }
  return strings.seatShieldNotifyTitleEvent;
}

export function useSeatShieldNotifications(
  strings: LocaleStrings,
  onUnreadChange?: (unread: number, critical: number) => void,
  onListItems?: (items: SecureNotificationItem[]) => void,
): void {
  const seenRef = useRef<Set<number>>(loadSeen());
  const stringsRef = useRef(strings);
  stringsRef.current = strings;

  const poll = useCallback(async () => {
    if (!nativeBridgeAvailable()) return;
    try {
      const resp = await runtimeRequest<Record<string, unknown>>("seat.shield.events", {
        after_sequence: 0,
        limit: 40,
      });
      if (!resp.ok || !resp.data) return;
      const events = mapSeatShieldEvents(resp.data as Record<string, unknown>);
      let touched = false;
      for (const event of events) {
        if (event.sequence <= 0) continue;
        if (seenRef.current.has(event.sequence)) continue;
        if (event.severity !== "high" && event.severity !== "critical" && event.severity !== "medium") {
          seenRef.current.add(event.sequence);
          touched = true;
          continue;
        }
        await postSecureNotification({
          title: titleForEvent(stringsRef.current, event),
          message: event.summary,
          severity: severityForEvent(event),
          source: "seat_shield",
          actionHint: "shield",
        });
        seenRef.current.add(event.sequence);
        touched = true;
      }
      if (touched) {
        saveSeen(seenRef.current);
      }
      const list = await listSecureNotifications();
      onUnreadChange?.(list.unreadCount, list.criticalCount);
      onListItems?.(list.items);
    } catch (error) {
      // Runtime may still be starting; the next poll retries.
      console.warn("seat shield notification poll failed", error);
    }
  }, [onUnreadChange, onListItems]);

  useEffect(() => {
    void poll();
    const id = window.setInterval(() => void poll(), POLL_MS);
    return () => window.clearInterval(id);
  }, [poll]);
}
