/**
 * Seat-launcher session stage. Display gating only — Runtime still owns
 * PIN, consume, portal, pack, and OIDC. Not Tauri. No auto-open.
 */

export type LauncherStage =
  | "offline"
  | "pin_missing"
  | "pin_locked"
  | "pin"
  | "consume"
  | "channel_invalid"
  | "portal"
  | "spa"
  | "login"
  | "ready";

export const LAUNCHER_JOURNEY = [
  "pin",
  "consume",
  "portal",
  "spa",
  "login",
  "ready",
] as const;

export type LauncherJourneyStep = (typeof LAUNCHER_JOURNEY)[number];

export type JourneyMark = "done" | "current" | "ahead" | "blocked";

export type LauncherStageInput = {
  pin_issued: boolean;
  pin_unlocked?: boolean;
  pin_locked?: boolean;
  capsule_imported: boolean;
  entitlement_present?: boolean;
  channel_present?: boolean;
  channel_valid?: boolean;
  channel_up?: boolean;
  spa_state?: string;
  oidc_session?: string;
};

export function launcherStage(status: LauncherStageInput | null): LauncherStage {
  if (!status) {
    return "offline";
  }
  if (status.pin_locked) {
    return "pin_locked";
  }
  if (!status.pin_issued) {
    return "pin_missing";
  }
  if (!status.pin_unlocked) {
    return "pin";
  }
  if (!status.capsule_imported || !status.entitlement_present || !status.channel_present) {
    return "consume";
  }
  if (!status.channel_valid) {
    return "channel_invalid";
  }
  if (!status.channel_up) {
    return "portal";
  }
  if (status.spa_state !== "ready") {
    return "spa";
  }
  if (status.oidc_session !== "present") {
    return "login";
  }
  return "ready";
}

/** Consume (bound capsule + entitlement) is the license. Not PIN-only. */
export function seatLicenseReady(status: LauncherStageInput | null): boolean {
  return Boolean(status?.capsule_imported && status.entitlement_present);
}

/** One-shot portal bring-up is allowed only on this stage. Never auto-open. */
export function mayAutoBringUpPortal(stage: LauncherStage): boolean {
  return stage === "portal";
}

/** Mutation result is enough to leave the portal step. Do not wait for a stale status poll. */
export function notePortalBroughtUp<T extends LauncherStageInput & { portal_hostmap?: string }>(
  status: T,
  hostmap?: string,
): T {
  const portal_hostmap = hostmap === "applied" || status.portal_hostmap === "applied"
    ? "applied"
    : status.portal_hostmap;
  return {
    ...status,
    channel_valid: true,
    channel_up: true,
    ...(portal_hostmap ? { portal_hostmap } : {}),
  };
}

/** Which journey step the rail should treat as the focus. */
export function journeyFocus(stage: LauncherStage): LauncherJourneyStep | null {
  if (stage === "offline") {
    return null;
  }
  if (stage === "pin_missing" || stage === "pin_locked" || stage === "pin") {
    return "pin";
  }
  if (stage === "consume" || stage === "channel_invalid") {
    return "consume";
  }
  if (stage === "portal") {
    return "portal";
  }
  if (stage === "spa") {
    return "spa";
  }
  if (stage === "login") {
    return "login";
  }
  return "ready";
}

export function journeyMark(step: LauncherJourneyStep, stage: LauncherStage): JourneyMark {
  const focus = journeyFocus(stage);
  if (stage === "pin_missing" || stage === "pin_locked" || stage === "channel_invalid") {
    const blocked: LauncherJourneyStep = stage === "channel_invalid" ? "consume" : "pin";
    if (step === blocked) {
      return "blocked";
    }
    return LAUNCHER_JOURNEY.indexOf(step) < LAUNCHER_JOURNEY.indexOf(blocked) ? "done" : "ahead";
  }
  if (!focus) {
    return "ahead";
  }
  const focusAt = LAUNCHER_JOURNEY.indexOf(focus);
  const stepAt = LAUNCHER_JOURNEY.indexOf(step);
  if (stepAt < focusAt) {
    return "done";
  }
  if (stepAt === focusAt) {
    return "current";
  }
  return "ahead";
}
