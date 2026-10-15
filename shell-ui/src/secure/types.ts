export type Availability =
  | "available"
  | "unavailable"
  | "partial"
  | "disabled";

export interface RuntimeErrorShape {
  code: string;
  message: string;
}

export interface RuntimeResponse<T> {
  protocol_version: number;
  id: string;
  ok: boolean;
  status: string;
  data: T | null;
  error?: RuntimeErrorShape | null;
}

export type DomainAvailability = "observed" | "partial" | "unavailable";

export interface DomainRecord<T> {
  availability: DomainAvailability;
  source: string;
  reason: string | null;
  data: T;
}

export interface RuntimeEvent {
  sequence: number;
  observed_at_epoch: number;
  domain: string;
  kind: "observed" | "changed" | "failed";
  summary: string;
}

export interface EventBatch {
  latest_sequence: number;
  truncated: boolean;
  events: RuntimeEvent[];
}

export interface SnapshotFreshness {
  age_ms: number;
  stale: boolean;
  limit_ms: number;
  reason: string;
  provenance: string;
}

export interface SystemSnapshot {
  sequence: number;
  observed_at_epoch: number;
  freshness: SnapshotFreshness;
  events?: RuntimeEvent[];
  events_truncated?: boolean;
  provenance: {
    collector: string;
    boot_id: string | null;
  };
  overall_status: "verified" | "degraded";
  runtime: {
    status: "ok";
    version: string;
    transport: "unix-peercred-v1";
    uptime_seconds: number;
    start_count?: number;
  };
  host: {
    hostname: string | null;
    kernel_release: string | null;
    boot_id: string | null;
    cpu_model?: string | null;
    cpu_count?: number | null;
    mem_total_kb?: number | null;
    mem_available_kb?: number | null;
    swap_total_kb?: number | null;
    swap_free_kb?: number | null;
    loadavg_1?: string | null;
    loadavg_5?: string | null;
    loadavg_15?: string | null;
  };
  network: {
    source: string;
    status: "connected" | "local" | "disconnected" | "unavailable";
    error: string | null;
    interfaces: NetworkInterface[];
    default_route: {
      interface: string | null;
      gateway: string | null;
    } | null;
    dns_servers: string[];
    dns_source: string;
  };
  power: {
    source: string;
    ac_online: boolean | null;
    battery_present: boolean;
    battery_percent: number | null;
    battery_state: string | null;
    highest_temperature_c: number | null;
    supplies?: Array<{
      name: string;
      kind: string;
      online: boolean | null;
      capacity: number | null;
      state: string | null;
      charge_now_uah?: number | null;
      charge_full_uah?: number | null;
      charge_full_design_uah?: number | null;
      voltage_now_uv?: number | null;
      current_now_ua?: number | null;
      manufacturer?: string | null;
      model?: string | null;
      technology?: string | null;
    }>;
    thermal_zones?: Array<{
      name: string;
      kind: string;
      temperature_c: number | null;
    }>;
    status: "observed" | "unavailable";
  };
  security: {
    source: string;
    secure_boot: boolean | null;
    dm_verity_root: boolean;
    root_read_only: boolean | null;
    kernel_lockdown: string | null;
    status: "verified" | "degraded";
    reasons: string[];
  };
  identity?: {
    source: string;
    kiosk_user_observed: boolean;
    maint_user_observed: boolean;
    ssh_allow_users: string[];
    password_authentication: boolean | null;
    permit_root_login: boolean | null;
    pubkey_authentication: boolean | null;
    kbd_interactive_authentication: boolean | null;
    authentication_methods: string | null;
    kiosk_authorized_keys_present: boolean;
    maint_authorized_keys_present: boolean;
    temporary_sudoers_present: boolean;
  };
  watchdog?: {
    source: string;
    present: boolean;
    devices?: Array<{
      name: string;
      identity: string | null;
      state: string | null;
      timeout_seconds: number | null;
    }>;
    status: "observed" | "unavailable";
  };
  services: DomainRecord<{
    units: Array<{
      name: string;
      observed: boolean;
      state?: string;
      result?: string;
      ran?: boolean;
      nrestarts?: number | null;
    }>;
  }>;
  wifi: DomainRecord<{
    interfaces: string[];
    associations?: Array<{
      interface: string;
      ssid: string;
      bssid?: string | null;
      frequency_mhz?: number | null;
      signal_dbm?: number | null;
      rx_mbps?: number | null;
      tx_mbps?: number | null;
    }>;
    radios?: Array<{
      name: string;
      kind: string;
      soft_blocked: boolean;
      hard_blocked: boolean;
      state: number;
    }>;
  }>;
  display: DomainRecord<{
    connectors: Array<{
      name: string;
      status: string;
      enabled?: string;
      dpms?: string;
      modes?: string[];
      connector_id?: number | null;
    }>;
    driver?: string | null;
    pci_id?: string | null;
    backlights?: Array<{
      name: string;
      brightness: number | null;
      max_brightness: number | null;
    }>;
  }>;
  input: DomainRecord<{
    event_nodes: number;
    devices?: Array<{
      node: string;
      name: string;
      has_key?: boolean;
      has_rel?: boolean;
      has_abs?: boolean;
      has_rel_wheel?: boolean;
    }>;
    rel_wheel_observed?: boolean;
    uinput_observed?: boolean;
  }>;
  audio: DomainRecord<{
    cards: number;
    devices?: Array<{ id: string; name: string }>;
    playback_name?: string | null;
    observed_percent?: number | null;
  }>;
  storage: DomainRecord<{
    root_fstype: string | null;
    root_read_only: boolean | null;
    root_total_bytes?: number | null;
    root_used_bytes?: number | null;
    root_available_bytes?: number | null;
    block_devices: number;
    devices?: Array<{ name: string; size_bytes: number | null }>;
  }>;
  usb?: DomainRecord<{
    devices: Array<{
      name: string;
      vendor_id?: string | null;
      product_id?: string | null;
      speed?: string | null;
      manufacturer?: string | null;
      product?: string | null;
    }>;
  }>;
  locale: DomainRecord<{
    locale: string | null;
    timezone: string | null;
    ntp_synchronized?: boolean | null;
    ntp_server?: string | null;
    ntp_last_sync_epoch?: number | null;
    ntp_stratum?: number | null;
    ntp_jitter_ms?: number | null;
  }>;
  capabilities: CapabilityStatus[];
}

export interface NetworkInterface {
  name: string;
  kind: "loopback" | "ethernet" | "wifi" | "other";
  operstate: string;
  carrier: boolean | null;
  mac_address: string | null;
  ipv4_addresses: string[];
  ipv6_addresses: string[];
  speed_mbps?: number | null;
  duplex?: string | null;
}

export interface CapabilityStatus {
  id: string;
  availability: Availability;
  source: string;
  reason: string;
}

export interface WifiAccessPoint {
  ssid: string;
  bssid: string;
  signal_dbm: number | null;
  signal_percent: number;
  frequency_mhz: number | null;
  security: string;
}

export interface WifiScanResult {
  interface: string;
  source: "iw";
  observed: true;
  access_points: WifiAccessPoint[];
}

export interface NetworkConfigureInput {
  interface: string;
  method: "dhcp" | "static";
  address?: string;
  prefix?: number;
  gateway?: string;
  dns: string[];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isNullableType(value: unknown, type: "string" | "number" | "boolean"): boolean {
  return value === null || typeof value === type;
}

export function isSystemSnapshot(value: unknown): value is SystemSnapshot {
  if (!isRecord(value)) return false;
  const runtime = value.runtime;
  const host = value.host;
  const network = value.network;
  const power = value.power;
  const security = value.security;
  if (
    typeof value.sequence !== "number" ||
    typeof value.observed_at_epoch !== "number" ||
    !isRecord(value.freshness) ||
    typeof value.freshness.age_ms !== "number" ||
    typeof value.freshness.stale !== "boolean" ||
    typeof value.freshness.limit_ms !== "number" ||
    typeof value.freshness.reason !== "string" ||
    typeof value.freshness.provenance !== "string" ||
    !isRecord(value.provenance) ||
    typeof value.provenance.collector !== "string" ||
    !isNullableType(value.provenance.boot_id, "string") ||
    !["verified", "degraded"].includes(String(value.overall_status)) ||
    !isRecord(runtime) ||
    runtime.status !== "ok" ||
    typeof runtime.version !== "string" ||
    runtime.transport !== "unix-peercred-v1" ||
    typeof runtime.uptime_seconds !== "number" ||
    !isRecord(host) ||
    !isNullableType(host.hostname, "string") ||
    !isNullableType(host.kernel_release, "string") ||
    !isNullableType(host.boot_id, "string") ||
    !isRecord(network) ||
    typeof network.source !== "string" ||
    !["connected", "local", "disconnected", "unavailable"].includes(
      String(network.status),
    ) ||
    !Array.isArray(network.interfaces) ||
    !network.interfaces.every(isNetworkInterface) ||
    !isNullableType(network.error, "string") ||
    !isDefaultRoute(network.default_route) ||
    !Array.isArray(network.dns_servers) ||
    !network.dns_servers.every((item) => typeof item === "string") ||
    typeof network.dns_source !== "string" ||
    !isRecord(power) ||
    typeof power.source !== "string" ||
    !isNullableType(power.ac_online, "boolean") ||
    typeof power.battery_present !== "boolean" ||
    !isNullableType(power.battery_percent, "number") ||
    !isNullableType(power.battery_state, "string") ||
    !isNullableType(power.highest_temperature_c, "number") ||
    !["observed", "unavailable"].includes(String(power.status)) ||
    !isRecord(security) ||
    typeof security.source !== "string" ||
    !isNullableType(security.secure_boot, "boolean") ||
    typeof security.dm_verity_root !== "boolean" ||
    !isNullableType(security.root_read_only, "boolean") ||
    !isNullableType(security.kernel_lockdown, "string") ||
    !["verified", "degraded"].includes(String(security.status)) ||
    !Array.isArray(security.reasons) ||
    !security.reasons.every((item) => typeof item === "string") ||
    !Array.isArray(value.capabilities) ||
    !value.capabilities.every(isCapabilityStatus) ||
    !isDomainRecord(value.services) ||
    !isDomainRecord(value.wifi) ||
    !isDomainRecord(value.display) ||
    !isDomainRecord(value.input) ||
    !isDomainRecord(value.audio) ||
    !isDomainRecord(value.storage) ||
    (value.usb !== undefined && !isDomainRecord(value.usb)) ||
    !isDomainRecord(value.locale)
  ) {
    return false;
  }
  return true;
}

function isDomainRecord(value: unknown): boolean {
  return (
    isRecord(value) &&
    ["observed", "partial", "unavailable"].includes(String(value.availability)) &&
    typeof value.source === "string" &&
    isNullableType(value.reason, "string") &&
    value.data !== undefined
  );
}

export interface WatchPayload extends EventBatch {
  snapshot: SystemSnapshot;
}

export function isWatchPayload(value: unknown): value is WatchPayload {
  return isEventBatch(value) && isRecord(value) && isSystemSnapshot(value.snapshot);
}

export function isEventBatch(value: unknown): value is EventBatch {
  if (!isRecord(value) || typeof value.latest_sequence !== "number" || typeof value.truncated !== "boolean" || !Array.isArray(value.events)) {
    return false;
  }
  return value.events.every((item) => {
    return (
      isRecord(item) &&
      typeof item.sequence === "number" &&
      typeof item.observed_at_epoch === "number" &&
      typeof item.domain === "string" &&
      ["observed", "changed", "failed"].includes(String(item.kind)) &&
      typeof item.summary === "string"
    );
  });
}

function isNetworkInterface(value: unknown): value is NetworkInterface {
  return (
    isRecord(value) &&
    typeof value.name === "string" &&
    ["loopback", "ethernet", "wifi", "other"].includes(String(value.kind)) &&
    typeof value.operstate === "string" &&
    isNullableType(value.carrier, "boolean") &&
    isNullableType(value.mac_address, "string") &&
    Array.isArray(value.ipv4_addresses) &&
    value.ipv4_addresses.every((item) => typeof item === "string") &&
    Array.isArray(value.ipv6_addresses) &&
    value.ipv6_addresses.every((item) => typeof item === "string")
  );
}

function isDefaultRoute(value: unknown): boolean {
  return (
    value === null ||
    (isRecord(value) &&
      isNullableType(value.interface, "string") &&
      isNullableType(value.gateway, "string"))
  );
}

function isCapabilityStatus(value: unknown): value is CapabilityStatus {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    ["available", "unavailable", "partial", "disabled"].includes(
      String(value.availability),
    ) &&
    typeof value.source === "string" &&
    typeof value.reason === "string"
  );
}
