import { useCallback, useEffect, useRef, useState } from "react";
import { runtimeRequest } from "./nativeTransport";
import {
  isSystemSnapshot,
  isWatchPayload,
  type RuntimeEvent,
  type SystemSnapshot,
} from "./types";

const WATCH_WAIT_MS = 4_000;
const WATCH_TIMEOUT_MS = 6_000;
const WATCH_RETRY_MS = 1_000;
const WATCH_ERROR_MAX_MS = 30_000;

export interface SnapshotState {
  snapshot: SystemSnapshot | null;
  events: RuntimeEvent[];
  staleMs: number | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

function freshnessAgeMs(snapshot: SystemSnapshot): number {
  return Math.max(0, Date.now() - snapshot.observed_at_epoch * 1000);
}

function applyLocalFreshness(snapshot: SystemSnapshot): SystemSnapshot {
  const ageMs = freshnessAgeMs(snapshot);
  const stale = ageMs > snapshot.freshness.limit_ms;
  return {
    ...snapshot,
    freshness: {
      ...snapshot.freshness,
      age_ms: ageMs,
      stale,
      reason: stale ? "age_exceeded" : snapshot.freshness.reason,
    },
  };
}

export function useSystemSnapshot(): SnapshotState {
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [events, setEvents] = useState<RuntimeEvent[]>([]);
  const [staleMs, setStaleMs] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const active = useRef(true);
  const inFlight = useRef(false);
  const watching = useRef(false);
  const failures = useRef(0);
  const timer = useRef<number | null>(null);
  const lastSequence = useRef(0);

  const schedule = useCallback((delay: number, run: () => void): void => {
    if (timer.current !== null) window.clearTimeout(timer.current);
    timer.current = window.setTimeout(run, delay);
  }, []);

  const acceptSnapshot = useCallback((candidate: SystemSnapshot): void => {
    const current = applyLocalFreshness(candidate);
    lastSequence.current = Math.max(lastSequence.current, current.sequence);
    setSnapshot(current);
    setStaleMs(current.freshness.stale ? current.freshness.age_ms : null);
    if (current.events && current.events.length > 0) {
      setEvents((existing) => [...existing, ...current.events!].slice(-32));
    }
  }, []);

  const refresh = useCallback(async (): Promise<void> => {
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      const response = await runtimeRequest<SystemSnapshot>(
        "system.snapshot",
        { after_sequence: lastSequence.current },
        10_000,
      );
      if (!response.ok || !response.data) {
        throw new Error(response.error?.message ?? `Runtime status: ${response.status}`);
      }
      if (!isSystemSnapshot(response.data)) {
        throw new Error("Runtime returned an invalid system.snapshot payload");
      }
      if (!active.current) return;
      failures.current = 0;
      acceptSnapshot(response.data);
      setError(null);
    } catch (caught) {
      if (!active.current) return;
      failures.current += 1;
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      inFlight.current = false;
      if (active.current) setLoading(false);
    }
  }, [acceptSnapshot]);

  useEffect(() => {
    active.current = true;
    const watch = async (): Promise<void> => {
      if (watching.current) return;
      watching.current = true;
      try {
        const response = await runtimeRequest<unknown>(
          "system.watch",
          { after_sequence: lastSequence.current, wait_ms: WATCH_WAIT_MS },
          WATCH_TIMEOUT_MS,
        );
        if (!response.ok || !response.data || !isWatchPayload(response.data)) {
          throw new Error(response.error?.message ?? "Runtime watch payload is invalid");
        }
        const payload = response.data;
        if (!active.current) return;
        failures.current = 0;
        if (payload.events.length > 0) {
          setEvents((existing) => [...existing, ...payload.events].slice(-32));
          window.dispatchEvent(
            new CustomEvent("empiieros-event", { detail: payload }),
          );
        }
        acceptSnapshot(payload.snapshot);
        setError(null);
      } catch (caught) {
        if (!active.current) return;
        failures.current += 1;
        setError(caught instanceof Error ? caught.message : String(caught));
        await refresh();
      } finally {
        watching.current = false;
        if (active.current) setLoading(false);
      }
      if (!active.current) return;
      const delay = failures.current === 0
        ? 0
        : Math.min(WATCH_RETRY_MS * 2 ** Math.min(failures.current, 3), WATCH_ERROR_MAX_MS);
      schedule(delay, () => {
        void watch();
      });
    };
    void watch();

    const onVisibility = (): void => {
      if (document.visibilityState === "visible") {
        schedule(0, () => {
          void watch();
        });
      }
    };
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      active.current = false;
      document.removeEventListener("visibilitychange", onVisibility);
      if (timer.current !== null) window.clearTimeout(timer.current);
    };
  }, [acceptSnapshot, refresh, schedule]);

  return { snapshot, events, staleMs, loading, error, refresh };
}
