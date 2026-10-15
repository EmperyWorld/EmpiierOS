import { fill } from "../format";
import { localizedValue, type LocaleStrings } from "../locale";
import type { SystemSnapshot } from "../types";

export type TrayLanguage = "ru" | "en";

export function trayLanguage(language: string): TrayLanguage {
  return language.toLocaleLowerCase().startsWith("ru") ? "ru" : "en";
}

export function trayLanguageLabel(language: string): string {
  return trayLanguage(language).toUpperCase();
}

export function trayNetworkLabel(
  snapshot: SystemSnapshot | null,
  strings: LocaleStrings,
): string {
  if (!snapshot) {
    return strings.refreshing;
  }
  const ssid = snapshot.wifi.data.associations?.[0]?.ssid?.trim();
  if (ssid) {
    return ssid;
  }
  const ethernet = snapshot.network.interfaces.find(
    (item) =>
      item.kind === "ethernet" && (item.operstate === "up" || item.carrier === true),
  );
  if (ethernet) {
    return ethernet.name;
  }
  return localizedValue(strings, snapshot.network.status);
}

export function trayBatteryPercent(snapshot: SystemSnapshot | null): number | null {
  if (snapshot?.power.battery_present && snapshot.power.battery_percent != null) {
    return Math.round(snapshot.power.battery_percent);
  }
  return null;
}

export function trayVolumePercent(snapshot: SystemSnapshot | null): number | null {
  const percent = snapshot?.audio.data.observed_percent;
  return percent == null ? null : Math.round(percent);
}

export function trayBatteryLabel(
  snapshot: SystemSnapshot | null,
  strings: LocaleStrings,
): string {
  const percent = trayBatteryPercent(snapshot);
  if (percent != null) {
    return fill(strings.volumeNow, { percent });
  }
  if (snapshot?.power.ac_online) {
    return strings.pluggedIn;
  }
  return snapshot ? strings.onBattery : "—";
}

export function trayBatteryTitle(
  snapshot: SystemSnapshot | null,
  strings: LocaleStrings,
): string {
  if (!snapshot) {
    return strings.battery;
  }
  const parts = [trayBatteryLabel(snapshot, strings)];
  if (snapshot.power.battery_state) {
    parts.push(localizedValue(strings, snapshot.power.battery_state));
  }
  if (snapshot.power.ac_online) {
    parts.push(strings.pluggedIn);
  }
  return parts.join(" · ");
}

export function traySoundLabel(
  snapshot: SystemSnapshot | null,
  strings: LocaleStrings,
): string {
  const percent = snapshot?.audio.data.observed_percent;
  if (percent == null) {
    return snapshot?.audio.data.playback_name ?? "—";
  }
  return fill(strings.volumeNow, { percent: Math.round(percent) });
}

export function trayClockLabel(
  now: Date,
  language: string,
  timeZone: string | null,
): string {
  const locale = trayLanguage(language) === "ru" ? "ru-RU" : "en-US";
  const options: Intl.DateTimeFormatOptions = {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  };
  if (timeZone) {
    try {
      return now.toLocaleTimeString(locale, { ...options, timeZone });
    } catch {
      /* seat timezone string is informational; fall back to local clock */
    }
  }
  return now.toLocaleTimeString(locale, options);
}

export function powerActionAvailable(snapshot: SystemSnapshot | null): boolean {
  return (
    snapshot?.capabilities.some(
      (item) => item.id === "system.power" && item.availability === "available",
    ) === true
  );
}
