/**
 * Capsule-gated OTA — Shell confirm + status helpers.
 * Confirm token must match Runtime `APPLY_OTA`. Secrets never enter IPC.
 */

export const CONFIRM_APPLY_OTA = "APPLY_OTA";

export type OtaNextAction = "fix_gates" | "check" | "install" | "current";

export type OtaStatusData = {
  ota_gate?: string;
  pin_unlocked?: boolean;
  license_ok?: boolean;
  allow_fofline?: boolean;
  device_keys_ok?: boolean;
  hardware_bound?: boolean;
  oidc_session?: string;
  portal_up?: boolean;
  portal_hostmap?: string;
  artifact_pk?: string;
  ota_helper?: string;
  ota_staged?: boolean;
  ota_apply?: string;
  ota_applied_build_id?: string | null;
  ota_staged_build_id?: string | null;
  ota_next_action?: OtaNextAction | string;
  detail?: string;
};

export function otaGatesClear(status: OtaStatusData | null | undefined): boolean {
  return status?.ota_gate === "ok";
}

export function buildOtaApplyParams(typedConfirm: string): Record<string, string> {
  if (typedConfirm !== CONFIRM_APPLY_OTA) {
    throw new Error("ota_confirm_mismatch");
  }
  return { confirm: CONFIRM_APPLY_OTA };
}

export function otaNextAction(status: OtaStatusData | null | undefined): OtaNextAction {
  const raw = status?.ota_next_action;
  if (raw === "fix_gates" || raw === "check" || raw === "install" || raw === "current") {
    return raw;
  }
  if (!otaGatesClear(status)) {
    return "fix_gates";
  }
  if (status?.ota_apply === "ready" || status?.ota_apply === "os_before_payload") {
    return "install";
  }
  if (status?.ota_apply === "applied") {
    return "current";
  }
  return "check";
}
