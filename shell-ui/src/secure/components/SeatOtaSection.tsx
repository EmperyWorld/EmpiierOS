import { useCallback, useEffect, useState } from "react";
import { TextField } from "../design/controls";
import { HostAppFact } from "../design/HostAppShell";
import { nativeBridgeAvailable, runtimeRequest } from "../nativeTransport";
import { localizedValue, type LocaleStrings } from "../locale";
import {
  CONFIRM_APPLY_OTA,
  buildOtaApplyParams,
  otaGatesClear,
  otaNextAction,
  type OtaStatusData,
} from "../../session/seatOta";

type Props = {
  strings: LocaleStrings;
};

function chipState(ok: boolean): string {
  return ok ? "connected" : "unavailable";
}

function otaErrorText(strings: LocaleStrings, code: string | null | undefined): string {
  if (!code) {
    return strings.unknown;
  }
  return localizedValue(strings, code);
}

function buildLabel(value: string | null | undefined, empty: string): string {
  const trimmed = typeof value === "string" ? value.trim() : "";
  return trimmed || empty;
}

export function SeatOtaSection({ strings }: Props) {
  const bridgeAvailable = nativeBridgeAvailable();
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<OtaStatusData | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirm, setConfirm] = useState("");

  const pullStatus = useCallback(
    async (opts?: { clearFeedback?: boolean }) => {
      if (!bridgeAvailable) {
        setError(strings.otaBridgeMissing);
        return;
      }
      setBusy(true);
      if (opts?.clearFeedback) {
        setError(null);
        setMessage(null);
      }
      try {
        const response = await runtimeRequest<OtaStatusData>("launcher.ota_status", {}, 10_000);
        if (!response.ok) {
          setStatus(null);
          setError(otaErrorText(strings, response.error?.message ?? response.status));
          return;
        }
        setStatus(response.data ?? null);
        if (opts?.clearFeedback) {
          setMessage(strings.otaStatusOk);
        }
      } catch (err) {
        setStatus(null);
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
    [bridgeAvailable, strings],
  );

  useEffect(() => {
    void pullStatus({ clearFeedback: false });
  }, [pullStatus]);

  const requestCapsule = async () => {
    if (!bridgeAvailable) {
      setError(strings.otaBridgeMissing);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.ota_request",
        {},
        900_000,
      );
      if (!response.ok) {
        setError(otaErrorText(strings, response.error?.message ?? response.status));
        await pullStatus();
        return;
      }
      const stagedId =
        typeof response.data?.ota_app_build_id === "string"
          ? response.data.ota_app_build_id
          : null;
      setMessage(
        stagedId
          ? strings.otaRequestOkWithBuild.replace("{build}", stagedId)
          : strings.otaRequestOk,
      );
      await pullStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const applyUpdate = async () => {
    if (!bridgeAvailable) {
      setError(strings.otaBridgeMissing);
      return;
    }
    if (confirm !== CONFIRM_APPLY_OTA) {
      setError(strings.otaConfirmMismatch);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const params = buildOtaApplyParams(confirm);
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.ota_apply",
        params,
        30_000,
      );
      if (!response.ok) {
        setError(otaErrorText(strings, response.error?.message ?? response.status));
        return;
      }
      const buildId =
        typeof response.data?.ota_app_build_id === "string"
          ? response.data.ota_app_build_id
          : status?.ota_staged_build_id ?? strings.unknown;
      const reloadShell = response.data?.ota_reload_shell === true;
      setMessage(
        (reloadShell ? strings.otaApplyOkReload : strings.otaApplyOkRestart).replace(
          "{build}",
          String(buildId),
        ),
      );
      setConfirm("");
      if (reloadShell) {
        window.setTimeout(() => {
          window.location.reload();
        }, 1200);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
      await pullStatus();
    }
  };

  const gatesOk = otaGatesClear(status);
  const next = otaNextAction(status);
  const oidcOk = status?.oidc_session === "present";
  const hostmapOk = status?.portal_hostmap === "applied";
  const pkOk = status?.artifact_pk === "present";
  const helperOk = status?.ota_helper === "present";
  const appliedBuild = buildLabel(status?.ota_applied_build_id ?? null, strings.otaBuildNone);
  const stagedBuild = buildLabel(status?.ota_staged_build_id ?? null, strings.otaBuildNone);
  const showInstall = next === "install";
  const nextLead =
    next === "fix_gates"
      ? strings.otaNextFixGates
      : next === "install"
        ? strings.otaNextInstall
        : next === "current"
          ? strings.otaNextCurrent
          : strings.otaNextCheck;

  return (
    <div className="host-app-section seat-ota-section" aria-label={strings.otaTitle}>
      <div className="seat-ota-block seat-ota-now">
        <h3 className="host-app-section-title">{strings.otaNowTitle}</h3>
        <div className="seat-ota-progress">
          <HostAppFact label={strings.otaInstalledBuild} value={appliedBuild} />
          <HostAppFact label={strings.otaDownloadedBuild} value={stagedBuild} />
          <HostAppFact
            label={strings.otaChipApply}
            value={otaErrorText(strings, status?.ota_apply ?? strings.unknown)}
          />
        </div>
        <p className="seat-ota-next" data-state={next} role="status">
          {nextLead}
        </p>
      </div>

      <div className="seat-ota-block">
        <h3 className="host-app-section-title">{strings.otaPrereq}</h3>
        <div className="host-app-status-row">
          <span className="secure-status" data-state={chipState(Boolean(status?.license_ok))}>
            {strings.otaChipLicense}
          </span>
          <span className="secure-status" data-state={chipState(Boolean(status?.device_keys_ok))}>
            {strings.otaChipDevice}
          </span>
          <span className="secure-status" data-state={chipState(oidcOk)}>
            {strings.otaChipOidc}
          </span>
          <span
            className="secure-status"
            data-state={chipState(Boolean(status?.portal_up) && hostmapOk)}
          >
            {strings.otaChipPortal}
          </span>
          <span className="secure-status" data-state={chipState(pkOk)}>
            {strings.otaChipArtifactPk}
          </span>
          <span className="secure-status" data-state={chipState(helperOk)}>
            {strings.otaChipHelper}
          </span>
        </div>
      </div>

      {message ? <p className="secure-message">{message}</p> : null}
      {error ? <p className="secure-error">{error}</p> : null}

      <div className="secure-actions seat-ota-primary-actions">
        <button
          type="button"
          className={next === "check" || next === "fix_gates" ? "seat-ota-primary" : undefined}
          disabled={!bridgeAvailable || busy || !gatesOk}
          onClick={() => void requestCapsule()}
        >
          {busy && next !== "install" ? strings.otaRequesting : strings.otaRequest}
        </button>
        <button
          type="button"
          disabled={!bridgeAvailable || busy}
          onClick={() => void pullStatus({ clearFeedback: true })}
        >
          {busy ? strings.refreshing : strings.otaRefresh}
        </button>
      </div>

      {showInstall ? (
        <div className="seat-ota-block seat-ota-install" aria-label={strings.otaApplySpoiler}>
          <h3 className="host-app-section-title">{strings.otaApplySpoiler}</h3>
          <p className="secure-detail">
            {strings.otaInstallLead.replace("{build}", stagedBuild)}
          </p>
          <div className="seat-ota-apply">
            <TextField
              label={strings.otaConfirmLabel}
              value={confirm}
              onChange={setConfirm}
              disabled={busy}
            />
            <div className="secure-actions">
              <button
                type="button"
                className="seat-ota-primary"
                disabled={!bridgeAvailable || busy || confirm !== CONFIRM_APPLY_OTA}
                onClick={() => void applyUpdate()}
              >
                {busy ? strings.otaApplying : strings.otaApply}
              </button>
            </div>
          </div>
        </div>
      ) : (
        <p className="secure-detail seat-ota-install-hint">{strings.otaInstallHiddenHint}</p>
      )}
    </div>
  );
}
