/**
 * EmpiierOS Seat Shield — Shell session helpers.
 * Twin: security.integrity (P84).
 */

import type { LocaleStrings } from "../secure/locale";

export type SeatShieldMode = "observe_only" | "armed" | "unavailable";

export interface SeatShieldStatusView {
  ok: boolean;
  product: string;
  mode: SeatShieldMode | string;
  actEnabled: boolean;
  autoActProtected: boolean;
  autoQuarantineHash: boolean;
  heuristicsEnabled: boolean;
  openChains: number;
  openChainTeasers: SeatShieldChainTeaserView[];
  quarantineCount: number;
  watchPending: number;
  lastWatchDrained: number;
  cycleIntervalSecs: number;
  protectedPorts: number[];
  allowlistCount: number;
  integrityTwin: string;
  integrityTwinState: string;
  lastCycleEpoch: number;
  lastObserveBackend: string;
  openHighEvents: number;
  backend: string;
  blockedCount: number;
  blockedPeers: string[];
  connectionCount: number;
  connectionsTruncated: boolean;
  connectionsEpoch: number;
  connections: SeatShieldConnView[];
  detail: string;
  otaBaseline: string;
}

export interface SeatShieldFeedStatusView {
  present: boolean;
  version: string | null;
  digest: string | null;
  ipCount: number;
  hashCount: number;
  detail: string;
}

export interface SeatShieldPolicyView {
  allowlist: string[];
  allowlistCount: number;
  actEnabled: boolean;
  autoActProtected: boolean;
  autoQuarantineHash: boolean;
  heuristicsEnabled: boolean;
  protectedPorts: number[];
}

export interface SeatShieldConnView {
  remoteIp: string;
  localPort: number;
  remotePort: number;
  processName: string;
  pid: number;
  exePath?: string;
  riskMilli: number;
  rttMs?: number;
}

export interface SeatShieldChainTeaserView {
  ip: string;
  kind: string;
  chainId: string;
  riskMilli: number;
}

export interface SeatShieldEventView {
  kind: string;
  severity: string;
  summary: string;
  entityIp?: string;
  entityProcess?: string;
  entityPort?: number;
  explainWhy: string;
  explainAction: string;
  sequence: number;
  epoch: number;
  chainId?: string;
}

export interface SeatShieldJournalAnalyticsView {
  critical: number;
  high: number;
  medium: number;
  low: number;
  topKinds: { kind: string; count: number }[];
  topIps: { ip: string; count: number }[];
}

export interface SeatShieldJournalGroupView {
  key: string;
  count: number;
  latestSequence: number;
}

export type SeatShieldJournalGroupBy = "none" | "chain" | "kind" | "severity" | "entity_ip";

export interface SeatShieldJournalView {
  items: SeatShieldEventView[];
  truncated: boolean;
  journalLines: number;
  groups: SeatShieldJournalGroupView[];
  analytics?: SeatShieldJournalAnalyticsView;
}

/** Locale key fragment for mode badge (resolve via LocaleStrings). */
export function seatShieldModeLocaleKey(mode: string): string {
  switch (mode) {
    case "observe_only":
      return "seatShieldModeObserveOnly";
    case "armed":
      return "seatShieldModeArmed";
    case "unavailable":
      return "seatShieldModeUnavailable";
    default:
      return "";
  }
}

/** Locale key fragment for backend badge. */
export function seatShieldBackendLocaleKey(backend: string): string {
  switch (backend) {
    case "unavailable":
      return "seatShieldBackendUnavailable";
    case "recording":
      return "seatShieldBackendRecording";
    case "helper":
      return "seatShieldBackendHelper";
    case "proc_net_tcp":
      return "seatShieldBackendProc";
    case "probe_inet_diag":
      return "seatShieldBackendProbe";
    default:
      return "";
  }
}

/** @deprecated Prefer localized keys via seatShieldModeLocaleKey. */
export function formatSeatShieldModeBadge(mode: string): string {
  switch (mode) {
    case "observe_only":
      return "Observe-only";
    case "armed":
      return "Armed";
    case "unavailable":
      return "Unavailable";
    default:
      return mode;
  }
}

/** @deprecated Prefer localized keys via seatShieldBackendLocaleKey. */
export function formatSeatShieldBackendBadge(backend: string): string {
  switch (backend) {
    case "unavailable":
      return "No floor helper";
    case "recording":
      return "Recording (lab)";
    case "helper":
      return "Pinned helper";
    case "proc_net_tcp":
      return "Live /proc observe";
    default:
      return backend;
  }
}

/** Map Runtime `seat.shield.status` data object into a Shell view model. */
export function mapSeatShieldStatus(data: Record<string, unknown>): SeatShieldStatusView {
  const peersRaw = data.blocked_peers;
  const blockedPeers = Array.isArray(peersRaw)
    ? peersRaw.map((p) => String(p)).filter((s) => s.length > 0)
    : [];
  return {
    ok: true,
    product: String(data.product ?? "empiieros-seat-shield"),
    mode: String(data.mode ?? "unavailable"),
    actEnabled: Boolean(data.act_enabled),
    autoActProtected: Boolean(data.auto_act_protected),
    autoQuarantineHash: Boolean(data.auto_quarantine_hash),
    heuristicsEnabled: data.heuristics_enabled !== false,
    openChains: Number(data.open_chains ?? 0),
    openChainTeasers: mapSeatShieldChainTeasers(data),
    quarantineCount: Number(data.quarantine_count ?? 0),
    watchPending: Number(data.watch_pending ?? 0),
    lastWatchDrained: Number(data.last_watch_drained ?? 0),
    cycleIntervalSecs: Number(data.cycle_interval_secs ?? 30),
    protectedPorts: Array.isArray(data.protected_ports)
      ? data.protected_ports.map((p) => Number(p)).filter((n) => Number.isFinite(n))
      : [],
    allowlistCount: Number(data.allowlist_count ?? 0),
    integrityTwin: String(data.integrity_twin ?? "security.integrity"),
    integrityTwinState: String(data.integrity_twin_state ?? "disabled"),
    lastCycleEpoch: Number(data.last_cycle_epoch ?? 0),
    lastObserveBackend: String(data.last_observe_backend ?? "none"),
    openHighEvents: Number(data.open_high_events ?? 0),
    backend: String(data.backend ?? "unavailable"),
    blockedCount: Number(data.blocked_count ?? blockedPeers.length),
    blockedPeers,
    connectionCount: Number(data.connection_count ?? 0),
    connectionsTruncated: Boolean(data.connections_truncated),
    connectionsEpoch: Number(data.connections_epoch ?? 0),
    connections: mapSeatShieldConnections(data),
    detail: String(data.detail ?? ""),
    otaBaseline: String(data.ota_baseline ?? "missing"),
  };
}

export function mapSeatShieldFeedStatus(data: Record<string, unknown>): SeatShieldFeedStatusView {
  return {
    present: data.present === true,
    version: data.version != null ? String(data.version) : null,
    digest: data.digest != null ? String(data.digest) : null,
    ipCount: Number(data.ip_count ?? 0),
    hashCount: Number(data.hash_count ?? 0),
    detail: String(data.detail ?? ""),
  };
}

export function mapSeatShieldPolicy(data: Record<string, unknown>): SeatShieldPolicyView {
  const allowRaw = data.allowlist;
  const allowlist = Array.isArray(allowRaw)
    ? allowRaw.map((ip) => String(ip)).filter((s) => s.length > 0)
    : [];
  return {
    allowlist,
    allowlistCount: Number(data.allowlist_count ?? allowlist.length),
    actEnabled: Boolean(data.act_enabled),
    autoActProtected: Boolean(data.auto_act_protected),
    autoQuarantineHash: Boolean(data.auto_quarantine_hash),
    heuristicsEnabled: data.heuristics_enabled !== false,
    protectedPorts: Array.isArray(data.protected_ports)
      ? data.protected_ports.map((p) => Number(p)).filter((n) => Number.isFinite(n))
      : [],
  };
}

export function otaBaselineLocaleKey(status: string): string {
  switch (status) {
    case "ok":
      return "seatShieldOtaBaselineOk";
    case "mismatch":
      return "seatShieldOtaBaselineMismatch";
    case "stale":
      return "seatShieldOtaBaselineStale";
    default:
      return "seatShieldOtaBaselineMissing";
  }
}

export function mapSeatShieldConnections(data: Record<string, unknown>): SeatShieldConnView[] {
  const raw = data.connections;
  if (!Array.isArray(raw)) return [];
  return raw.map((item) => {
    const row = (item ?? {}) as Record<string, unknown>;
    return {
      remoteIp: String(row.remote_ip ?? ""),
      localPort: Number(row.local_port ?? 0),
      remotePort: Number(row.remote_port ?? 0),
      processName: String(row.process_name ?? ""),
      pid: Number(row.pid ?? 0),
      exePath: row.exe_path != null ? String(row.exe_path) : undefined,
      riskMilli: Number(row.risk_milli ?? 0),
      rttMs: row.rtt_ms != null ? Number(row.rtt_ms) : undefined,
    };
  });
}

export function mapSeatShieldChainTeasers(
  data: Record<string, unknown>,
): SeatShieldChainTeaserView[] {
  const raw = data.open_chain_teasers;
  if (!Array.isArray(raw)) return [];
  return raw.map((item) => {
    const row = (item ?? {}) as Record<string, unknown>;
    return {
      ip: String(row.ip ?? ""),
      kind: String(row.kind ?? ""),
      chainId: String(row.chain_id ?? ""),
      riskMilli: Number(row.risk_milli ?? 0),
    };
  });
}

function mapOneEvent(row: Record<string, unknown>): SeatShieldEventView {
  const severityRaw = row.severity;
  const severity =
    typeof severityRaw === "string"
      ? severityRaw
      : severityRaw != null
        ? String(severityRaw)
        : "";
  return {
    kind: String(row.kind ?? ""),
    severity,
    summary: String(row.summary ?? ""),
    entityIp: row.entity_ip != null ? String(row.entity_ip) : undefined,
    entityProcess: row.entity_process != null ? String(row.entity_process) : undefined,
    entityPort: row.entity_port != null ? Number(row.entity_port) : undefined,
    explainWhy: String(row.explain_why ?? ""),
    explainAction: String(row.explain_action ?? ""),
    sequence: Number(row.sequence ?? 0),
    epoch: Number(row.epoch ?? 0),
    chainId: row.chain_id != null ? String(row.chain_id) : undefined,
  };
}

/**
 * Map Runtime `seat.shield.events` data.
 * Runtime returns `items`; cycle responses use `events` — accept both.
 */
/** Locale key fragment for known S4 event kinds (empty = show raw kind). */
export function seatShieldEventKindLocaleKey(kind: string): string {
  switch (kind) {
    case "network_suspicious_port":
      return "seatShieldKindSuspiciousPort";
    case "possible_port_scan":
      return "seatShieldKindPortScan";
    case "kernel_taint":
      return "seatShieldKindKernelTaint";
    case "act_applied":
      return "seatShieldKindActApplied";
    case "ioc_hash_match":
      return "seatShieldKindIocHash";
    case "peer_probe_protected_service":
      return "seatShieldKindPeerProbe";
    case "suspicious_exec_location":
      return "seatShieldKindSuspiciousExec";
    case "ioc_peer_match":
      return "seatShieldKindIocPeer";
    case "act_preview":
      return "seatShieldKindActPreview";
    case "act_denied":
      return "seatShieldKindActDenied";
    case "policy_suppressed":
      return "seatShieldKindPolicySuppressed";
    case "backend_unavailable":
      return "seatShieldKindBackendUnavailable";
    case "file_scan_hit":
      return "seatShieldKindFileScanHit";
    case "file_scan_denied":
      return "seatShieldKindFileScanDenied";
    case "file_quarantined":
      return "seatShieldKindFileQuarantined";
    case "file_restored":
      return "seatShieldKindFileRestored";
    case "file_watch_seen":
      return "seatShieldKindFileWatchSeen";
    case "chain_scan_then_probe":
      return "seatShieldKindChainScanProbe";
    case "chain_ioc_peer_probe":
      return "seatShieldKindChainIocProbe";
    case "chain_watch_ioc_live":
      return "seatShieldKindChainWatchLive";
    case "chain_tmp_exec_net":
      return "seatShieldKindChainTmpExec";
    case "chain_multi_signal":
      return "seatShieldKindChainMulti";
    case "heuristic_reconnect_burst":
      return "seatShieldKindReconnectBurst";
    case "ota_component_mismatch":
      return "seatShieldKindOtaMismatch";
    case "ota_capsule_staged":
      return "seatShieldKindOtaStaged";
    default:
      return "";
  }
}

export interface SeatShieldQuarantineItemView {
  sha256: string;
  size: number;
  epoch: number;
  reason: string;
  origBasename: string;
}

export function mapSeatShieldQuarantineList(
  data: Record<string, unknown>,
): SeatShieldQuarantineItemView[] {
  const raw = data.items;
  if (!Array.isArray(raw)) return [];
  return raw.map((item) => {
    const row = (item ?? {}) as Record<string, unknown>;
    return {
      sha256: String(row.sha256 ?? ""),
      size: Number(row.size ?? 0),
      epoch: Number(row.epoch ?? 0),
      reason: String(row.reason ?? ""),
      origBasename: String(row.orig_basename ?? "unknown"),
    };
  });
}

export function mapSeatShieldEvents(data: Record<string, unknown>): SeatShieldEventView[] {
  const raw = data.items ?? data.events;
  if (!Array.isArray(raw)) return [];
  return raw.map((item) => mapOneEvent((item ?? {}) as Record<string, unknown>));
}

export function mapSeatShieldJournal(data: Record<string, unknown>): SeatShieldJournalView {
  const analyticsRaw =
    data.analytics && typeof data.analytics === "object"
      ? (data.analytics as Record<string, unknown>)
      : null;
  const bySev =
    analyticsRaw?.by_severity && typeof analyticsRaw.by_severity === "object"
      ? (analyticsRaw.by_severity as Record<string, unknown>)
      : null;
  const topKindsRaw = Array.isArray(analyticsRaw?.by_kind_top) ? analyticsRaw.by_kind_top : [];
  const topIpsRaw = Array.isArray(analyticsRaw?.top_ips) ? analyticsRaw.top_ips : [];
  const groupsRaw = Array.isArray(data.groups) ? data.groups : [];
  return {
    items: mapSeatShieldEvents(data),
    truncated: Boolean(data.truncated),
    journalLines: Number(data.journal_lines ?? 0),
    groups: groupsRaw.map((item) => {
      const row = (item ?? {}) as Record<string, unknown>;
      return {
        key: String(row.key ?? ""),
        count: Number(row.count ?? 0),
        latestSequence: Number(row.latest_sequence ?? 0),
      };
    }),
    analytics: analyticsRaw
      ? {
          critical: Number(bySev?.critical ?? 0),
          high: Number(bySev?.high ?? 0),
          medium: Number(bySev?.medium ?? 0),
          low: Number(bySev?.low ?? 0),
          topKinds: topKindsRaw.map((item) => {
            const row = (item ?? {}) as Record<string, unknown>;
            return { kind: String(row.kind ?? ""), count: Number(row.count ?? 0) };
          }),
          topIps: topIpsRaw.map((item) => {
            const row = (item ?? {}) as Record<string, unknown>;
            return { ip: String(row.ip ?? ""), count: Number(row.count ?? 0) };
          }),
        }
      : undefined,
  };
}

/** Runtime `seat.shield.observe` notes are honest machine strings — map for operators. */
export function formatSeatShieldObserveMessage(
  data: Record<string, unknown>,
  strings: LocaleStrings,
): string {
  const note = String(data.note ?? "").trim();
  const connections = Number(data.connection_count ?? 0);
  const newEvents = Array.isArray(data.events) ? data.events.length : 0;
  const applied = Array.isArray(data.applied_acts) ? data.applied_acts.length : 0;
  const deniedNote = note.includes("auto-act denied");
  const previewNote = note.includes("preview-only") || note.includes("did not mutate floor");
  const appliedNote = note.includes("auto-act applied") || applied > 0;

  if (appliedNote) {
    return strings.seatShieldObserveNoteApplied
      .replace("{applied}", String(applied))
      .replace("{connections}", String(connections))
      .replace("{events}", String(newEvents));
  }
  if (deniedNote) {
    return strings.seatShieldObserveNoteDenied;
  }
  if (previewNote || note.length === 0) {
    return strings.seatShieldObserveNotePreview
      .replace("{connections}", String(connections))
      .replace("{events}", String(newEvents));
  }
  return strings.seatShieldObserveNoteGeneric.replace("{detail}", note);
}
