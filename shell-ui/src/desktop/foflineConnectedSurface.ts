/**
 * Copied from EliteEW Desktop foflineConnectedSurface.ts and adapted.
 * Source: powered_by_emperyworld/shared/web/frontend/src/pages/Desktop/shell/foflineConnectedSurface.ts
 * Profile: shared/apps/_profiles/fofline-connected.yaml
 *
 * FoFline Start/catalog surface. Fail-closed. Not cloud-full. Not air-gap.
 */

export const FOFLINE_CONNECTED_PROFILE_ID = "fofline-connected";

export const FOFLINE_CONNECTED_APP_COMPONENTS: ReadonlySet<string> = new Set([
  "NotificationsApp",
  "ProjectAboutApp",
  "ClipboardManager",
  "MusicPlayer",
  "AppStore",
  "AssistantApp",
  "EWDeskApp",
  "PoemmaApp",
  "EwaApp",
  "ContentManagerPro",
  "UGS",
  "UGSHubApp",
  "UserProfile",
  "VoiceStudio",
  "GAFChatApp",
  "VideoCallApp",
]);

export function intersectFoflineConnectedSurface(
  enabled: ReadonlySet<string>,
): ReadonlySet<string> {
  const out = new Set<string>();
  for (const id of enabled) {
    if (FOFLINE_CONNECTED_APP_COMPONENTS.has(id)) {
      out.add(id);
    }
  }
  return out;
}

export function isFoflineConnectedApp(appComponent: string): boolean {
  return FOFLINE_CONNECTED_APP_COMPONENTS.has(appComponent);
}
