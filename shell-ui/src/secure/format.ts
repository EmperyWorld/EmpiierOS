import type { LocaleStrings } from "./locale";

export function fill(
  template: string,
  values: Record<string, string | number>,
): string {
  return template.replace(/\{(\w+)\}/g, (_, key: string) =>
    Object.prototype.hasOwnProperty.call(values, key) ? String(values[key]) : "",
  );
}

export function formatUptime(seconds: number, strings: LocaleStrings): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return fill(strings.uptimeHoursMinutes, { hours, minutes });
  }
  if (minutes > 0) {
    return fill(strings.uptimeMinutes, { minutes });
  }
  return fill(strings.uptimeSeconds, { seconds });
}

export function formatYesNo(value: boolean | null, strings: LocaleStrings): string {
  if (value === null) return strings.unknown;
  return value ? strings.yes : strings.no;
}

export function formatOnOff(value: boolean | null, strings: LocaleStrings): string {
  if (value === null) return strings.unknown;
  return value ? strings.on : strings.off;
}

export function formatLockdown(value: string | null, strings: LocaleStrings): string {
  if (!value || value === "none" || value === "unknown") {
    return strings.off;
  }
  return strings.on;
}

export function serviceUnitReady(unit: {
  state?: string;
  result?: string;
  ran?: boolean;
}): boolean {
  if (unit.state === "active") {
    return true;
  }
  return unit.state === "inactive" && unit.result === "success" && unit.ran === true;
}

export function formatServiceUnit(
  unit: {
    observed: boolean;
    state?: string;
    result?: string;
    ran?: boolean;
    nrestarts?: number | null;
  },
  strings: LocaleStrings,
): string {
  const parts = [
    unit.state ?? (unit.observed ? strings.ready : strings.unavailable),
  ];
  if (unit.result) {
    parts.push(unit.result);
  }
  if (unit.ran === true) {
    parts.push(strings.serviceRan);
  } else if (unit.ran === false) {
    parts.push(strings.serviceNotRun);
  }
  if (unit.nrestarts != null) {
    parts.push(fill(strings.serviceRestarts, { count: unit.nrestarts }));
  }
  return parts.join(" ");
}

export function formatBytes(value: number | null | undefined): string | null {
  if (value == null || value < 0) {
    return null;
  }
  if (value >= 10_000_000_000) {
    return `${Math.round(value / 1_000_000_000)} GB`;
  }
  if (value >= 1_000_000_000) {
    return `${(value / 1_000_000_000).toFixed(1)} GB`;
  }
  if (value >= 1_000_000) {
    return `${Math.round(value / 1_000_000)} MB`;
  }
  return `${value} B`;
}

export function formatMbps(value: number | null | undefined): string | null {
  if (value == null || value <= 0) {
    return null;
  }
  return `${value} Mbps`;
}

export function formatMhz(value: number | null | undefined): string | null {
  if (value == null || value <= 0) {
    return null;
  }
  return `${value} MHz`;
}

export function formatDbm(value: number | null | undefined): string | null {
  if (value == null) {
    return null;
  }
  return `${value} dBm`;
}

export function formatConnector(item: {
  name: string;
  status: string;
  enabled?: string;
  dpms?: string;
  modes?: string[];
}): string {
  const parts = [`${item.name}: ${item.status}`];
  if (item.enabled && item.enabled !== "unknown") {
    parts.push(item.enabled);
  }
  if (item.dpms && item.dpms !== "unknown") {
    parts.push(`dpms=${item.dpms}`);
  }
  if (item.modes && item.modes.length > 0) {
    parts.push(item.modes.join("/"));
  }
  return parts.join(" ");
}

export function brightnessPercent(
  current: number | null,
  max: number | null,
): number | null {
  if (current === null || max === null || max <= 0) {
    return null;
  }
  return Math.round((current / max) * 100);
}
