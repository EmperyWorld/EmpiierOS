import type { Workspace } from "../secure/components/Taskbar";
import type { LocaleStrings } from "../secure/locale";

/** OS seat modules on the host desktop. Not EliteEW pack apps. */
export type HostModuleId =
  | "launcher"
  | "shield"
  | "browser"
  | "system"
  | "network"
  | "diagnostics"
  | "ota"
  | "audio"
  | "events";

export interface HostModule {
  id: HostModuleId;
  icon: string;
  opens: "window" | "workspace";
  workspace?: Workspace;
  windowApp?: "NotificationsApp" | "MusicPlayer";
}

export const FOFLINE_HOST_MODULES: readonly HostModule[] = [
  { id: "launcher", icon: "launcher.png", opens: "window" },
  { id: "shield", icon: "host-shield.png", opens: "workspace", workspace: "shield" },
  { id: "browser", icon: "host-browser.png", opens: "workspace", workspace: "browser" },
  { id: "system", icon: "host-system.png", opens: "workspace", workspace: "system" },
  { id: "ota", icon: "host-ota.png", opens: "workspace", workspace: "ota" },
  { id: "network", icon: "host-network.png", opens: "workspace", workspace: "network" },
  {
    id: "diagnostics",
    icon: "host-diag.png",
    opens: "workspace",
    workspace: "diagnostics",
  },
  { id: "audio", icon: "host-audio.png", opens: "window", windowApp: "MusicPlayer" },
  {
    id: "events",
    icon: "host-events.png",
    opens: "window",
    windowApp: "NotificationsApp",
  },
];

/** Seat floor stays up before PIN / consume. Product tiles wait for the license. */
export function isSeatFloorModule(id: HostModuleId): boolean {
  return id === "launcher" || id === "network";
}

export function visibleHostModules(
  licensed: boolean,
  modules: readonly HostModule[] = FOFLINE_HOST_MODULES,
): readonly HostModule[] {
  if (licensed) {
    return modules;
  }
  return modules.filter((item) => isSeatFloorModule(item.id));
}

export function hostModuleIconSrc(file: string): string {
  return `./product-chrome/${file}`;
}

export function hostModuleTitle(strings: LocaleStrings, id: HostModuleId): string {
  if (id === "launcher") return strings.foflineActivatorTitle;
  if (id === "shield") return strings.seatShield;
  if (id === "browser") return strings.seatBrowser;
  if (id === "system") return strings.workspaceSystem;
  if (id === "network") return strings.workspaceNetwork;
  if (id === "diagnostics") return strings.workspaceDiagnostics;
  if (id === "ota") return strings.workspaceOta;
  if (id === "audio") return strings.foflineLocalAudio;
  return strings.foflineLocalEvents;
}
