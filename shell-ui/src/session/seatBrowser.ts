/**
 * EmpiierOS Seat Browser — Shell session helpers (S3 host overlay).
 */

export interface SeatBrowserTabView {
  id: number;
  host: string;
  url: string;
}

export interface SeatBrowserDownloadView {
  id: number;
  name: string;
  host: string;
  bytes: number;
  phase: string;
}

export interface SeatBrowserStatusView {
  product: string;
  surface: string;
  host: string;
  policyMode: string;
  policyFresh: boolean;
  isDefault: boolean;
  allowCount: number;
  denyCount: number;
  allowHosts: string[];
  denyHosts: string[];
  allowHostsObserved: boolean;
  tabCount: number;
  tabCap: number;
  activeTab: number | null;
  tabs: SeatBrowserTabView[];
  downloadCount: number;
  downloadCap: number;
  downloads: SeatBrowserDownloadView[];
  camera: boolean;
  microphone: boolean;
  detail: string;
}

export function normalizePolicyHost(raw: string): string | null {
  let value = raw.trim().toLowerCase();
  if (!value) {
    return null;
  }
  if (/\s/.test(value) || value.includes("@")) {
    return null;
  }
  if (value.includes("://")) {
    try {
      value = new URL(value).hostname.toLowerCase();
    } catch {
      return null;
    }
  }
  if (!value || value.length > 253) {
    return null;
  }
  return value;
}

export function normalizeAllowHosts(hosts: readonly string[]): string[] {
  const out: string[] = [];
  const seen = new Set<string>();
  for (const raw of hosts) {
    const host = normalizePolicyHost(raw);
    if (!host || seen.has(host)) {
      continue;
    }
    seen.add(host);
    out.push(host);
  }
  return out;
}

export function parsePolicyHostList(value: unknown): string[] | undefined {
  if (!Array.isArray(value)) {
    return undefined;
  }
  const out: string[] = [];
  const seen = new Set<string>();
  for (const item of value) {
    if (typeof item !== "string") {
      continue;
    }
    const trimmed = item.trim();
    if (!trimmed) {
      continue;
    }
    const key = trimmed.toLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    out.push(trimmed);
  }
  return out;
}

export function parseTabList(value: unknown): SeatBrowserTabView[] {
  if (!Array.isArray(value)) {
    return [];
  }
  const out: SeatBrowserTabView[] = [];
  const seen = new Set<number>();
  for (const item of value) {
    if (item == null || typeof item !== "object") {
      continue;
    }
    const row = item as Record<string, unknown>;
    const id = Number(row.id);
    if (!Number.isInteger(id) || id <= 0 || seen.has(id)) {
      continue;
    }
    seen.add(id);
    out.push({
      id,
      host: String(row.host ?? ""),
      url: String(row.url ?? ""),
    });
    if (out.length >= 3) {
      break;
    }
  }
  return out;
}

export function parseDownloadList(value: unknown): SeatBrowserDownloadView[] {
  if (!Array.isArray(value)) {
    return [];
  }
  const out: SeatBrowserDownloadView[] = [];
  const seen = new Set<number>();
  for (const item of value) {
    if (item == null || typeof item !== "object") {
      continue;
    }
    const row = item as Record<string, unknown>;
    const id = Number(row.id);
    if (!Number.isInteger(id) || id <= 0 || seen.has(id)) {
      continue;
    }
    seen.add(id);
    const name = String(row.name ?? "").trim();
    if (!name || name.includes("/") || name.includes("\\")) {
      continue;
    }
    out.push({
      id,
      name,
      host: String(row.host ?? ""),
      bytes: Number(row.bytes ?? 0),
      phase: String(row.phase ?? ""),
    });
    if (out.length >= 16) {
      break;
    }
  }
  return out;
}

export function mapSeatBrowserStatus(data: Record<string, unknown>): SeatBrowserStatusView {
  const allowHosts = parsePolicyHostList(data.allow);
  const denyHosts = parsePolicyHostList(data.deny);
  const tabs = parseTabList(data.tabs);
  const downloads = parseDownloadList(data.downloads);
  const activeRaw = data.active_tab;
  const activeTab =
    typeof activeRaw === "number" && Number.isInteger(activeRaw) && activeRaw > 0
      ? activeRaw
      : null;
  return {
    product: String(data.product ?? "empiieros-seat-browser"),
    surface: String(data.surface ?? "shell_overlay"),
    host: String(data.host ?? "webkit_gtk"),
    policyMode: String(data.policy_mode ?? "allowlist"),
    policyFresh: Boolean(data.policy_fresh),
    isDefault: Boolean(data.is_default),
    allowCount: Number(data.allow_count ?? allowHosts?.length ?? 0),
    denyCount: Number(data.deny_count ?? denyHosts?.length ?? 0),
    allowHosts: allowHosts ?? [],
    denyHosts: denyHosts ?? [],
    allowHostsObserved: allowHosts !== undefined,
    tabCount: Number(data.tab_count ?? tabs.length),
    tabCap: Number(data.tab_cap ?? 3),
    activeTab,
    tabs,
    downloadCount: Number(data.download_count ?? downloads.length),
    downloadCap: Number(data.download_cap ?? 16),
    downloads,
    camera: data.camera !== false,
    microphone: data.microphone !== false,
    detail: String(data.detail ?? ""),
  };
}
