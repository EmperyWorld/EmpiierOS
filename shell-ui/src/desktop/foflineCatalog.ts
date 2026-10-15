/**
 * FoFline catalog: app_component set copied from EliteEW DEFAULT_APPS
 * plus fofline-connected.yaml extras. Icons, cloud doors, and SPA mounts stripped.
 */

import { isFoflineConnectedApp } from "./foflineConnectedSurface";

export type FoflineLocalKind = "events" | "audio" | "about" | "catalog" | "pack";

export interface FoflineApp {
  id: string;
  appComponent: string;
  localKind: FoflineLocalKind;
}

const RAW_CATALOG: FoflineApp[] = [
  { id: "app-notifications", appComponent: "NotificationsApp", localKind: "events" },
  { id: "app-about-project", appComponent: "ProjectAboutApp", localKind: "about" },
  { id: "widget-clipboard", appComponent: "ClipboardManager", localKind: "pack" },
  { id: "app-music-player", appComponent: "MusicPlayer", localKind: "audio" },
  { id: "app-store", appComponent: "AppStore", localKind: "catalog" },
  { id: "app-assistant", appComponent: "AssistantApp", localKind: "pack" },
  { id: "app-ewdesk", appComponent: "EWDeskApp", localKind: "pack" },
  { id: "app-poemma", appComponent: "PoemmaApp", localKind: "pack" },
  { id: "app-ewa", appComponent: "EwaApp", localKind: "pack" },
  { id: "app-content", appComponent: "ContentManagerPro", localKind: "pack" },
  { id: "app-ugs", appComponent: "UGS", localKind: "pack" },
  { id: "app-ugs-hub", appComponent: "UGSHubApp", localKind: "pack" },
  { id: "app-user-profile", appComponent: "UserProfile", localKind: "pack" },
  { id: "app-voice-studio", appComponent: "VoiceStudio", localKind: "pack" },
  { id: "app-gaf-chat", appComponent: "GAFChatApp", localKind: "pack" },
  { id: "app-video-call", appComponent: "VideoCallApp", localKind: "pack" },
];

export const FOFLINE_CATALOG: FoflineApp[] = RAW_CATALOG.filter((item) =>
  isFoflineConnectedApp(item.appComponent),
);

/** OS host tiles only. Pack apps open inside FoFline after product.open. */
export function foflineHostTiles(apps: readonly FoflineApp[] = FOFLINE_CATALOG): FoflineApp[] {
  return apps.filter((item) => item.localKind !== "pack");
}

export function foflineAppByComponent(appComponent: string): FoflineApp | undefined {
  return FOFLINE_CATALOG.find((item) => item.appComponent === appComponent);
}

export function foflineGlyph(appComponent: string): string {
  const letter = appComponent.replace(/App$|Pro$|Manager$/, "").charAt(0);
  return letter || "F";
}
