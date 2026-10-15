import { DetailsSpoiler, StatusPill } from "../secure/design/controls";
import { localizedValue, type LocaleStrings } from "../secure/locale";
import {
  LAUNCHER_JOURNEY,
  journeyFocus,
  journeyMark,
  type LauncherJourneyStep,
  type LauncherStage,
} from "./foflineLauncherStage";

/** Visible on the path tab so an OTA shell apply can be confirmed on the seat. */
export const LAUNCHER_SHELL_BUILD = "ota-app-1";

const JOURNEY_STEP_LABEL: Record<LauncherJourneyStep, keyof LocaleStrings> = {
  pin: "foflineActivatorStepUnlock",
  consume: "foflineActivatorStepActivate",
  portal: "foflineActivatorStepPortal",
  spa: "foflineActivatorStepPack",
  login: "foflineActivatorStepSignIn",
  ready: "foflineActivatorStepOpen",
};

function journeyStepTitle(step: LauncherJourneyStep, strings: LocaleStrings): string {
  return strings[JOURNEY_STEP_LABEL[step]] as string;
}

export type LauncherStatusView = {
  vehicle: string;
  tauri: boolean;
  fingerprint: string;
  fingerprint_bound: boolean;
  pin_issued: boolean;
  pin_unlocked?: boolean;
  pin_locked?: boolean;
  capsule_imported: boolean;
  product_id?: string;
  capsule_source?: string;
  entitlement_present?: boolean;
  channel_present?: boolean;
  channel_valid?: boolean;
  channel_up?: boolean;
  allow_fofline?: boolean | null;
  spa_state?: string;
  spa_code?: string;
  spa_app_build_id?: string | null;
  product_open: string;
  oidc_session?: string;
  portal_hostmap?: string;
  detail: string;
};

export function launcherStageView(
  stage: LauncherStage,
  strings: LocaleStrings,
): { label: string; hint: string; state: string } {
  const focus = journeyFocus(stage);
  const label = focus ? journeyStepTitle(focus, strings) : strings.foflineActivatorTitle;
  if (stage === "pin_missing") {
    return { label, hint: strings.foflineActivatorPinMissing, state: "disabled" };
  }
  if (stage === "pin_locked") {
    return { label, hint: strings.foflineActivatorPinLocked, state: "disabled" };
  }
  if (stage === "pin") {
    return { label, hint: strings.foflineActivatorStagePin, state: "observed" };
  }
  if (stage === "consume") {
    return { label, hint: strings.foflineActivatorStageConsume, state: "observed" };
  }
  if (stage === "channel_invalid") {
    return { label, hint: strings.foflineActivatorStageChannelInvalid, state: "disabled" };
  }
  if (stage === "portal") {
    return { label, hint: strings.foflineActivatorStagePortal, state: "observed" };
  }
  if (stage === "spa") {
    return { label, hint: strings.foflineActivatorStageSpa, state: "observed" };
  }
  if (stage === "login") {
    return { label, hint: strings.foflineActivatorStageLogin, state: "observed" };
  }
  if (stage === "ready") {
    return { label, hint: strings.foflineActivatorStageReady, state: "verified" };
  }
  return { label, hint: strings.foflineActivatorStageOffline, state: "unavailable" };
}

export function FoflineLauncherJourney({
  stage,
  strings,
}: {
  stage: LauncherStage;
  strings: LocaleStrings;
}) {
  return (
    <ol className="fofline-journey" aria-label={strings.foflineActivatorJourneyLabel}>
      {LAUNCHER_JOURNEY.map((step, index) => {
        const mark = journeyMark(step, stage);
        return (
          <li key={step} data-mark={mark} aria-current={mark === "current" ? "step" : undefined}>
            <span className="fofline-journey-index">{mark === "done" ? "✓" : index + 1}</span>
            <span>{journeyStepTitle(step, strings)}</span>
          </li>
        );
      })}
    </ol>
  );
}

export function FoflinePinPad({
  busy,
  label,
  onClear,
  onDigit,
}: {
  busy: boolean;
  label: string;
  onClear: () => void;
  onDigit: (digit: string) => void;
}) {
  return (
    <div className="fofline-pin-pad" role="group" aria-label={label}>
      {["1", "2", "3", "4", "5", "6", "7", "8", "9", "C", "0"].map((key) => (
        <button
          key={key}
          type="button"
          disabled={busy}
          onClick={() => {
            if (key === "C") {
              onClear();
              return;
            }
            onDigit(key);
          }}
        >
          {key}
        </button>
      ))}
    </div>
  );
}

export function FoflineActivatorLab({
  status,
  strings,
  productAvailability,
  launcherReason,
  launcherId,
  pinUnlocked,
  pinLocked,
  needsUnlock,
  pinEntry,
  importPath,
  airCode,
  busy,
  bridgeAvailable,
  showConsumeFields,
  onImportPath,
  onAirCode,
  onRefresh,
  onImport,
  onAir,
  onSpa,
  onPortal,
  onLogin,
  onLogout,
  onOpen,
  onUnlock,
  windowMode,
  onToggleWindow,
}: {
  status: LauncherStatusView | null;
  strings: LocaleStrings;
  productAvailability: string;
  launcherReason?: string;
  launcherId?: string;
  pinUnlocked: boolean;
  pinLocked: boolean;
  needsUnlock: boolean;
  pinEntry: string;
  importPath: string;
  airCode: string;
  busy: boolean;
  bridgeAvailable: boolean;
  showConsumeFields: boolean;
  onImportPath: (value: string) => void;
  onAirCode: (value: string) => void;
  onRefresh: () => void;
  onImport: () => void;
  onAir: () => void;
  onSpa: () => void;
  onPortal: () => void;
  onLogin: () => void;
  onLogout: () => void;
  onOpen: () => void;
  onUnlock: () => void;
  windowMode: "kiosk" | "windowed";
  onToggleWindow: () => void;
}) {
  return (
    <DetailsSpoiler summary={strings.foflineActivatorLabTools}>
      <p className="secure-detail">{strings.foflineActivatorLabSpaNote}</p>
      <p className="secure-detail">{strings.windowModeServiceHint}</p>
      <div className="fofline-floor">
        <StatusPill
          label={strings.windowModeServiceLabel}
          value={windowMode === "kiosk" ? strings.kioskMode : strings.windowedMode}
          state={windowMode === "kiosk" ? "connected" : "local"}
        />
        <button type="button" disabled={!bridgeAvailable || busy} onClick={onToggleWindow}>
          {windowMode === "kiosk" ? strings.windowedMode : strings.kioskMode}
        </button>
      </div>
      <StatusPill
        value={status?.vehicle ?? strings.foflineActivatorTitle}
        state={status && !status.tauri ? "verified" : "disabled"}
      />
      <p>
        {strings.foflineActivatorVehicle}: {status?.vehicle ?? strings.unknown}
      </p>
      <p>
        {strings.foflineActivatorFingerprint}: {status?.fingerprint ?? strings.unknown}
      </p>
      <p>
        {strings.foflineActivatorPin}:{" "}
        {pinUnlocked
          ? strings.foflineActivatorPinUnlocked
          : pinLocked
            ? strings.foflineActivatorPinLocked
            : status?.pin_issued
              ? strings.foflineActivatorPinIssued
              : strings.foflineActivatorPinMissing}
        {status?.fingerprint_bound ? ` · ${strings.foflineActivatorPinBound}` : ""}
      </p>
      <p>
        {strings.foflineActivatorCapsule}:{" "}
        {status?.capsule_imported ? strings.yes : strings.foflineActivatorCapsuleMissing}
        {status?.capsule_imported && status.product_id ? ` · ${status.product_id}` : ""}
        {status?.capsule_imported && status.capsule_source ? ` · ${status.capsule_source}` : ""}
      </p>
      <p>
        {status?.entitlement_present
          ? strings.foflineActivatorEntitlement
          : strings.foflineActivatorInnerMissing}
        {status?.channel_present ? ` · ${strings.foflineActivatorChannel}` : ""}
        {status?.channel_valid ? ` · ${strings.foflineActivatorChannelValid}` : ""}
        {status?.channel_up ? ` · ${strings.foflineActivatorChannelUp}` : ""}
      </p>
      <p>
        {strings.foflineActivatorHostmap}:{" "}
        {status?.portal_hostmap === "applied"
          ? strings.foflineActivatorHostmapApplied
          : strings.foflineActivatorHostmapMissing}
      </p>
      <p>
        {status?.spa_state === "ready"
          ? strings.foflineActivatorSpaReady
          : strings.foflineActivatorSpaMissing}
        {status?.spa_app_build_id ? ` · ${status.spa_app_build_id}` : ""}
        {status?.spa_code ? ` · ${status.spa_code}` : ""}
      </p>
      <p>
        {status?.oidc_session === "present"
          ? strings.foflineActivatorOidcReady
          : strings.foflineActivatorOidcMissing}
      </p>
      <StatusPill value={localizedValue(strings, productAvailability)} state={productAvailability} />
      <p>
        {status?.product_open === "allowed" && status.oidc_session === "present"
          ? strings.foflineActivatorProductHint
          : strings.foflineActivatorProductHintLocked}
      </p>
      {pinUnlocked ? (
        showConsumeFields && (
          <>
            <label>
              {strings.foflineActivatorImportPath}
              <input
                value={importPath}
                disabled={busy}
                onChange={(event) => onImportPath(event.target.value)}
              />
            </label>
            <label>
              {strings.foflineActivatorAirCode}
              <input
                value={airCode}
                disabled={busy}
                autoComplete="off"
                spellCheck={false}
                onChange={(event) => onAirCode(event.target.value)}
              />
            </label>
          </>
        )
      ) : (
        <p>{strings.foflineActivatorConsumeLocked}</p>
      )}
      <div className="fofline-floor">
        <button type="button" disabled={!bridgeAvailable || busy} onClick={onRefresh}>
          {busy ? strings.refreshing : strings.foflineActivatorRefresh}
        </button>
        <button type="button" disabled={!bridgeAvailable || busy || !pinUnlocked} onClick={onImport}>
          {busy ? strings.foflineActivatorImporting : strings.foflineActivatorImport}
        </button>
        <button
          type="button"
          disabled={!bridgeAvailable || busy || !pinUnlocked || airCode.trim().length === 0}
          onClick={onAir}
        >
          {busy ? strings.foflineActivatorAiring : strings.foflineActivatorAir}
        </button>
        <button
          type="button"
          disabled={
            !bridgeAvailable ||
            busy ||
            !pinUnlocked ||
            !status?.allow_fofline ||
            !status?.channel_up
          }
          onClick={onSpa}
        >
          {busy ? strings.foflineActivatorSpaSyncing : strings.foflineActivatorSpaSync}
        </button>
        <button
          type="button"
          disabled={
            !bridgeAvailable ||
            busy ||
            !pinUnlocked ||
            !status?.channel_valid ||
            Boolean(status?.channel_up)
          }
          onClick={onPortal}
        >
          {busy ? strings.foflineActivatorPortalUpping : strings.foflineActivatorPortalUp}
        </button>
        <button
          type="button"
          disabled={
            !bridgeAvailable ||
            busy ||
            !pinUnlocked ||
            !status?.allow_fofline ||
            !status?.channel_up ||
            status?.oidc_session === "present"
          }
          onClick={onLogin}
        >
          {busy ? strings.foflineActivatorOidcStarting : strings.foflineActivatorOidcStart}
        </button>
        <button
          type="button"
          disabled={!bridgeAvailable || busy || !pinUnlocked || status?.oidc_session !== "present"}
          onClick={onLogout}
        >
          {busy ? strings.foflineActivatorOidcLeaving : strings.foflineActivatorOidcLeave}
        </button>
        <button
          type="button"
          disabled={
            !bridgeAvailable ||
            busy ||
            !pinUnlocked ||
            status?.spa_state !== "ready" ||
            status?.product_open !== "allowed" ||
            status?.oidc_session !== "present"
          }
          onClick={onOpen}
        >
          {busy ? strings.foflineActivatorProductOpening : strings.foflineActivatorProductOpen}
        </button>
        <button
          type="button"
          disabled={!bridgeAvailable || busy || !needsUnlock || pinEntry.length !== 4}
          onClick={onUnlock}
        >
          {busy ? strings.foflineActivatorUnlocking : strings.foflineActivatorUnlock}
        </button>
      </div>
      {launcherId && launcherReason && (
        <p className="secure-detail">
          {localizedValue(strings, launcherId)}: {localizedValue(strings, launcherReason)}
        </p>
      )}
    </DetailsSpoiler>
  );
}
