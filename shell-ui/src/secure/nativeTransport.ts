import type { RuntimeResponse } from "./types";

declare global {
  interface Window {
    webkit?: {
      messageHandlers?: {
        empiieros?: {
          postMessage: (message: unknown) => void;
        };
      };
    };
  }
}

type PendingRequest = {
  resolve: (response: RuntimeResponse<unknown>) => void;
  reject: (error: Error) => void;
  timeout: number;
};

const pending = new Map<string, PendingRequest>();
let listening = false;
let sequence = 0;

function ensureListener(): void {
  if (listening) return;
  window.addEventListener("empiieros-response", (event: Event) => {
    const detail = (event as CustomEvent<unknown>).detail;
    if (!isRuntimeResponse(detail)) return;
    const request = pending.get(detail.id);
    if (!request) return;
    pending.delete(detail.id);
    window.clearTimeout(request.timeout);
    request.resolve(detail);
  });
  listening = true;
}

function nextId(): string {
  sequence = (sequence + 1) % Number.MAX_SAFE_INTEGER;
  const random =
    typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : `${Date.now().toString(36)}-${sequence.toString(36)}`;
  return `shell-${random}`;
}

function isRuntimeResponse(value: unknown): value is RuntimeResponse<unknown> {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Record<string, unknown>;
  return (
    candidate.protocol_version === 1 &&
    typeof candidate.id === "string" &&
    typeof candidate.ok === "boolean" &&
    typeof candidate.status === "string" &&
    Object.prototype.hasOwnProperty.call(candidate, "data")
  );
}

export function nativeBridgeAvailable(): boolean {
  return typeof window.webkit?.messageHandlers?.empiieros?.postMessage === "function";
}

export async function runtimeRequest<T>(
  operation: string,
  params: object = {},
  timeoutMs = 35_000,
): Promise<RuntimeResponse<T>> {
  ensureListener();
  const bridge = window.webkit?.messageHandlers?.empiieros;
  if (!bridge || typeof bridge.postMessage !== "function") {
    throw new Error("trusted native bridge is unavailable");
  }
  const id = nextId();
  return new Promise<RuntimeResponse<T>>((resolve, reject) => {
    const timeout = window.setTimeout(() => {
      pending.delete(id);
      reject(new Error(`runtime request timed out: ${operation}`));
    }, timeoutMs);
    pending.set(id, {
      resolve: (response) => resolve(response as RuntimeResponse<T>),
      reject,
      timeout,
    });
    try {
      bridge.postMessage({ id, op: operation, params });
    } catch (error) {
      pending.delete(id);
      window.clearTimeout(timeout);
      reject(error instanceof Error ? error : new Error(String(error)));
    }
  });
}
