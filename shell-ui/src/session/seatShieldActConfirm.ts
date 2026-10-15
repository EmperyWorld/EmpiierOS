/**
 * S3 Shell — confirm-gated Seat Shield acts (F8 sensitive-actions).
 * Operator must type confirmation strings; Shell never auto-injects them.
 */

export const CONFIRM_BLOCK_PEER = "BLOCK_PEER";
export const CONFIRM_UNBLOCK_PEER = "UNBLOCK_PEER";
export const CONFIRM_ENABLE_ACTS = "ENABLE_ACTS";
export const CONFIRM_ENABLE_QUARANTINE = "ENABLE_QUARANTINE";
export const CONFIRM_QUARANTINE_FILE = "QUARANTINE_FILE";
export const CONFIRM_RESTORE_QUARANTINE = "RESTORE_QUARANTINE";
export const CONFIRM_RESET_SEAT_SHIELD = "RESET_SEAT_SHIELD";

export type SeatShieldActAction = "preview_block" | "block_peer" | "unblock_peer";

export interface SeatShieldActParams {
  action: SeatShieldActAction;
  peerIp: string;
  /** Required for block_peer / unblock_peer — must match operator-typed confirm. */
  confirmation?: string;
}

/** Expected confirm token for a destructive act (empty for preview). */
export function expectedConfirmForAct(action: SeatShieldActAction): string {
  switch (action) {
    case "block_peer":
      return CONFIRM_BLOCK_PEER;
    case "unblock_peer":
      return CONFIRM_UNBLOCK_PEER;
    case "preview_block":
      return "";
    default:
      return "";
  }
}

/**
 * Build Runtime params — confirmation must already match the expected token.
 * Callers verify typed confirm before invoking this.
 */
export function buildSeatShieldActParams(input: SeatShieldActParams): Record<string, string> {
  const params: Record<string, string> = {
    action: input.action,
    peer_ip: input.peerIp,
  };
  const expected = expectedConfirmForAct(input.action);
  if (expected) {
    if (input.confirmation !== expected) {
      throw new Error("seat_shield_confirm_mismatch");
    }
    params.confirmation = expected;
  }
  return params;
}

/** Params for `seat.shield.policy.set` arming acts. */
export function buildEnableActsParams(
  typedConfirm: string,
  autoActProtected = false,
): Record<string, unknown> {
  if (typedConfirm !== CONFIRM_ENABLE_ACTS) {
    throw new Error("seat_shield_confirm_mismatch");
  }
  const params: Record<string, unknown> = {
    act_enabled: true,
    confirmation: CONFIRM_ENABLE_ACTS,
  };
  if (autoActProtected) {
    params.auto_act_protected = true;
  }
  return params;
}

/** Already-armed path: set protected auto-act without a silent toggle. */
export function buildSetAutoActProtectedParams(enabled: boolean): Record<string, unknown> {
  return { auto_act_protected: enabled };
}

/** Disarm does not require Runtime confirmation — Shell still gates on operator intent. */
export function buildDisableActsParams(): Record<string, unknown> {
  return { act_enabled: false };
}

export function buildEnableAutoQuarantineParams(
  typedConfirm: string,
): Record<string, unknown> {
  if (typedConfirm !== CONFIRM_ENABLE_QUARANTINE) {
    throw new Error("seat_shield_confirm_mismatch");
  }
  return {
    auto_quarantine_hash: true,
    confirmation: CONFIRM_ENABLE_QUARANTINE,
  };
}

export function buildDisableAutoQuarantineParams(): Record<string, unknown> {
  return { auto_quarantine_hash: false };
}

/** Observe-only heuristics kill-switch — no floor confirm. */
export function buildSetHeuristicsParams(enabled: boolean): Record<string, unknown> {
  return { heuristics_enabled: enabled };
}

/** Replace allowlist wholesale (validated server-side). */
export function buildSetAllowlistParams(allowlist: string[]): Record<string, unknown> {
  return { allowlist };
}

export function buildScanPathParams(path: string): Record<string, string> {
  return { path };
}

export function buildQuarantineFileParams(
  path: string,
  typedConfirm: string,
): Record<string, string> {
  if (typedConfirm !== CONFIRM_QUARANTINE_FILE) {
    throw new Error("seat_shield_confirm_mismatch");
  }
  return { path, confirmation: CONFIRM_QUARANTINE_FILE };
}

export function buildRestoreQuarantineParams(
  sha256: string,
  typedConfirm: string,
): Record<string, string> {
  if (typedConfirm !== CONFIRM_RESTORE_QUARANTINE) {
    throw new Error("seat_shield_confirm_mismatch");
  }
  return { sha256, confirmation: CONFIRM_RESTORE_QUARANTINE };
}

export function buildResetParams(typedConfirm: string): Record<string, string> {
  if (typedConfirm !== CONFIRM_RESET_SEAT_SHIELD) {
    throw new Error("seat_shield_confirm_mismatch");
  }
  return { confirmation: CONFIRM_RESET_SEAT_SHIELD };
}

/** Show Arm when acts are not yet enabled (helper may still be missing). */
export function seatShieldArmVisible(actEnabled: boolean): boolean {
  return !actEnabled;
}

/** Show Disarm when acts are enabled. */
export function seatShieldDisarmVisible(actEnabled: boolean): boolean {
  return actEnabled;
}

/**
 * UX rule: show Block/Unblock only when status.mode === "armed"
 * and status.backend === "helper".
 */
export function seatShieldActsVisible(mode: string, backend: string): boolean {
  return mode === "armed" && backend === "helper";
}
