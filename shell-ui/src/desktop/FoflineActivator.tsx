import { useEffect, useRef, useState } from "react";
import { StatusPill } from "../secure/design/controls";
import { HostAppFact, HostAppShell } from "../secure/design/HostAppShell";
import type { LocaleStrings } from "../secure/locale";
import { SeatClockSet } from "../secure/components/SeatClockSet";
import { nativeBridgeAvailable, runtimeRequest } from "../secure/nativeTransport";
import type { SystemSnapshot } from "../secure/types";
import {
  FoflineActivatorLab,
  FoflineLauncherJourney,
  FoflinePinPad,
  LAUNCHER_SHELL_BUILD,
  launcherStageView,
} from "./foflineLauncherChrome";
import { launcherStage, mayAutoBringUpPortal, notePortalBroughtUp } from "./foflineLauncherStage";

const FOFLINE_LOCAL_START =
  "fofline-spa://localhost/index.html?eliteewFoflineClient=1";

interface Props {
  snapshot: SystemSnapshot | null;
  strings: LocaleStrings;
  windowMode: "kiosk" | "windowed";
  onToggleWindow: () => Promise<void>;
}

interface LauncherStatus {
  vehicle: string;
  tauri: boolean;
  fingerprint: string;
  fingerprint_scheme: string;
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
  channel_code?: string;
  portal_hostmap?: string;
  wall_clock_ok?: boolean;
  allow_fofline?: boolean | null;
  spa_state?: string;
  spa_code?: string;
  spa_app_build_id?: string | null;
  product_open: string;
  oidc_session?: string;
  detail: string;
}

export function FoflineActivator({ snapshot, strings, windowMode, onToggleWindow }: Props) {
  const [status, setStatus] = useState<LauncherStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [importPath, setImportPath] = useState("/var/lib/empiieros-runtime/capsules/current.ecap");
  const [importMessage, setImportMessage] = useState<string | null>(null);
  const [airCode, setAirCode] = useState("");
  const [pinEntry, setPinEntry] = useState("");
  const portalAutoTried = useRef(false);
  const statusGen = useRef(0);
  const [tab, setTab] = useState<"path" | "status" | "lab">("path");
  const product = snapshot?.capabilities.find((item) => item.id === "product.open");
  const launcherCap = snapshot?.capabilities.find((item) => item.id === "launcher.status");
  const bridgeAvailable = nativeBridgeAvailable();
  const pinUnlocked = Boolean(status?.pin_unlocked);
  const pinLocked = Boolean(status?.pin_locked);
  const needsUnlock = Boolean(status?.pin_issued) && !pinUnlocked && !pinLocked;
  const stage = launcherStage(status);
  const view = launcherStageView(stage, strings);

  const refresh = async (opts?: {
    keepOnError?: boolean;
    keepPortalUp?: boolean;
  }): Promise<LauncherStatus | null> => {
    const keepOnError = Boolean(opts?.keepOnError);
    const gen = ++statusGen.current;
    if (!keepOnError) {
      setBusy(true);
      setError(null);
    }
    try {
      const response = await runtimeRequest<LauncherStatus>("launcher.status", {}, 5_000);
      if (!response.ok || !response.data) {
        throw new Error(response.error?.message ?? response.status);
      }
      if (gen !== statusGen.current) {
        return response.data;
      }
      const next =
        opts?.keepPortalUp && response.data.channel_up !== true
          ? notePortalBroughtUp(response.data)
          : response.data;
      setStatus(next);
      return next;
    } catch (caught) {
      if (!keepOnError && gen === statusGen.current) {
        setStatus(null);
        setError(caught instanceof Error ? caught.message : String(caught));
      }
      return null;
    } finally {
      if (!keepOnError && gen === statusGen.current) {
        setBusy(false);
      }
    }
  };

  const activateAir = async (): Promise<void> => {
    if (!pinUnlocked) {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      if (status?.wall_clock_ok === false) {
        throw new Error("wall_clock_required");
      }
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.air_activate",
        { code: airCode },
        180_000,
      );
      setAirCode("");
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
      await refresh();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  const unlockPin = async (): Promise<void> => {
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.pin_unlock",
        { pin: pinEntry },
        5_000,
      );
      setPinEntry("");
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
      await refresh();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  const syncSpa = async (): Promise<void> => {
    if (!pinUnlocked) {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.spa_sync",
        {},
        600_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
      await refresh();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  const bringUpPortal = async (): Promise<void> => {
    if (!pinUnlocked) {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.channel_up",
        {},
        45_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      if (response.data?.channel_up === true) {
        statusGen.current += 1;
        const hostmap =
          typeof response.data.portal_hostmap === "string"
            ? response.data.portal_hostmap
            : undefined;
        setStatus((prev) => (prev ? notePortalBroughtUp(prev, hostmap) : prev));
        if (hostmap !== "applied") {
          throw new Error("portal_hostmap_required");
        }
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
      await refresh({ keepPortalUp: response.data?.channel_up === true });
    } catch (caught) {
      const next = await refresh({ keepOnError: true });
      if (next?.channel_up) {
        setError(null);
      } else {
        setError(caught instanceof Error ? caught.message : String(caught));
      }
    } finally {
      setBusy(false);
    }
  };

  const startLogin = async (): Promise<void> => {
    if (!pinUnlocked) {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      if (status?.wall_clock_ok === false) {
        throw new Error("wall_clock_required");
      }
      let next = status;
      if (next?.portal_hostmap !== "applied") {
        const hostmap = await runtimeRequest<Record<string, unknown>>(
          "launcher.hostmap_apply",
          {},
          8_000,
        );
        if (!hostmap.ok) {
          throw new Error(hostmap.error?.message ?? hostmap.status);
        }
        next = await refresh({ keepOnError: true });
        if (next?.portal_hostmap !== "applied") {
          throw new Error("portal_hostmap_required");
        }
      }
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.oidc_start",
        {},
        20_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      const entrypoint =
        typeof response.data?.authorization_url === "string"
          ? response.data.authorization_url
          : "";
      if (!entrypoint.startsWith("https://auth.eliteew.ru/realms/empery/protocol/openid-connect/auth")) {
        throw new Error("entrypoint_not_allowlisted");
      }
      const surface = await runtimeRequest<Record<string, unknown>>(
        "shell.auth_surface",
        { action: "show", entrypoint },
        5_000,
      );
      if (!surface.ok) {
        throw new Error(surface.error?.message ?? surface.status);
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  const repairHostmap = async (): Promise<void> => {
    if (!pinUnlocked) {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.hostmap_apply",
        {},
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.foflineActivatorHostmapApplied,
      );
      await refresh({ keepOnError: true });
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  const endSession = async (): Promise<void> => {
    if (!pinUnlocked || status?.oidc_session !== "present") {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.oidc_logout",
        {},
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      const hideProduct = await runtimeRequest<Record<string, unknown>>(
        "shell.product_surface",
        { action: "hide" },
        5_000,
      );
      if (!hideProduct.ok) {
        throw new Error(hideProduct.error?.message ?? hideProduct.status);
      }
      await runtimeRequest<Record<string, unknown>>(
        "shell.auth_surface",
        { action: "hide" },
        5_000,
      );
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
      await refresh();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  const openProduct = async (): Promise<void> => {
    if (!pinUnlocked || status?.oidc_session !== "present") {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "product.open",
        { target: "fofline_local" },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      const entrypoint =
        typeof response.data?.entrypoint_url === "string"
          ? response.data.entrypoint_url
          : "";
      if (entrypoint !== FOFLINE_LOCAL_START) {
        throw new Error("entrypoint_not_allowlisted");
      }
      const surface = await runtimeRequest<Record<string, unknown>>(
        "shell.product_surface",
        { action: "show", entrypoint },
        5_000,
      );
      if (!surface.ok) {
        throw new Error(surface.error?.message ?? surface.status);
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
      await refresh();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  const importCapsule = async (): Promise<void> => {
    if (!pinUnlocked) {
      return;
    }
    setBusy(true);
    setError(null);
    setImportMessage(null);
    try {
      const response = await runtimeRequest<Record<string, unknown>>(
        "launcher.capsule_import",
        { path: importPath },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setImportMessage(
        typeof response.data?.detail === "string"
          ? response.data.detail
          : strings.operationApplied,
      );
      await refresh();
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    if (!bridgeAvailable) {
      return;
    }
    void refresh();
    const timer = window.setInterval(() => {
      void refresh({ keepOnError: true });
    }, 4000);
    return () => window.clearInterval(timer);
  }, [bridgeAvailable]);

  useEffect(() => {
    if (!bridgeAvailable) {
      return;
    }
    const onOidc = (event: Event): void => {
      const detail = (event as CustomEvent<{ ok?: boolean }>).detail;
      void refresh().then((next) => {
        if (!detail?.ok && next?.oidc_session !== "present") {
          setError(strings.foflineActivatorOidcFailed);
        }
      });
    };
    window.addEventListener("empiieros-oidc", onOidc);
    return () => window.removeEventListener("empiieros-oidc", onOidc);
  }, [bridgeAvailable, strings.foflineActivatorOidcFailed]);

  useEffect(() => {
    if (!bridgeAvailable || busy || portalAutoTried.current) {
      return;
    }
    if (!mayAutoBringUpPortal(stage)) {
      return;
    }
    portalAutoTried.current = true;
    void bringUpPortal();
  }, [bridgeAvailable, busy, stage]);

  return (
    <div className="fofline-activator host-app-root" data-shell-build={LAUNCHER_SHELL_BUILD}>
      <HostAppShell
        ariaLabel={strings.foflineActivatorTitle}
        tabs={[
          { id: "path", label: strings.hostAppTabLauncherPath },
          { id: "status", label: strings.hostAppTabLauncherStatus },
          { id: "lab", label: strings.hostAppTabLauncherLab },
        ]}
        activeTab={tab}
        onTabChange={(id) => setTab(id as "path" | "status" | "lab")}
        busy={busy}
        message={importMessage}
        error={error}
        statusChips={[
          { label: view.label, state: view.state },
          {
            label: pinUnlocked
              ? strings.foflineActivatorPinUnlocked
              : pinLocked
                ? strings.foflineActivatorPinLocked
                : status?.pin_issued
                  ? strings.foflineActivatorPinIssued
                  : strings.foflineActivatorPinMissing,
            state: pinUnlocked ? "connected" : pinLocked ? "degraded" : "local",
          },
          {
            label: status?.channel_up
              ? strings.foflineActivatorChannelUp
              : status?.channel_valid
                ? strings.foflineActivatorChannelValid
                : strings.foflineActivatorChannel,
            state: status?.channel_up
              ? "connected"
              : status?.channel_valid
                ? "local"
                : "unavailable",
          },
          {
            label:
              status?.spa_state === "ready"
                ? strings.foflineActivatorSpaReady
                : strings.foflineActivatorSpaMissing,
            state: status?.spa_state === "ready" ? "connected" : "unavailable",
          },
          {
            label:
              status?.wall_clock_ok === false
                ? strings.foflineActivatorClockRequired
                : status?.wall_clock_ok === true
                  ? strings.foflineActivatorClockOk
                  : strings.clock,
            state:
              status?.wall_clock_ok === false
                ? "degraded"
                : status?.wall_clock_ok === true
                  ? "connected"
                  : "local",
          },
        ]}
      >
        {tab === "path" ? (
          <div className="host-app-section">
            <FoflineLauncherJourney stage={stage} strings={strings} />
            {status?.wall_clock_ok !== true && <SeatClockSet strings={strings} />}
            {status && status.wall_clock_ok === false && (
              <p role="alert">{strings.foflineActivatorClockRequired}</p>
            )}
            <div className="fofline-activator-now">
              <StatusPill value={view.label} state={view.state} />
              <p>{view.hint}</p>
            </div>
            {stage === "pin" && (
              <>
                <div className="fofline-pin-slots" aria-hidden="true">
                  {[0, 1, 2, 3].map((slot) => (
                    <span key={slot} data-filled={pinEntry.length > slot ? "1" : "0"} />
                  ))}
                </div>
                <label className="fofline-pin-field">
                  {strings.foflineActivatorPinEntry}
                  <input
                    value={pinEntry}
                    disabled={busy}
                    inputMode="numeric"
                    autoComplete="off"
                    spellCheck={false}
                    maxLength={4}
                    onChange={(event) =>
                      setPinEntry(event.target.value.replace(/\D/g, "").slice(0, 4))
                    }
                  />
                </label>
                <FoflinePinPad
                  busy={busy}
                  label={strings.foflineActivatorPinEntry}
                  onClear={() => setPinEntry("")}
                  onDigit={(digit) => setPinEntry((current) => (current + digit).slice(0, 4))}
                />
              </>
            )}
            {stage === "consume" && pinUnlocked && (
              <>
                <label>
                  {strings.foflineActivatorImportPath}
                  <input
                    value={importPath}
                    disabled={busy}
                    onChange={(event) => setImportPath(event.target.value)}
                  />
                </label>
                <label>
                  {strings.foflineActivatorAirCode}
                  <input
                    value={airCode}
                    disabled={busy}
                    autoComplete="off"
                    spellCheck={false}
                    onChange={(event) => setAirCode(event.target.value)}
                  />
                </label>
              </>
            )}
            <div className="fofline-floor fofline-floor-primary">
              {stage === "pin" && (
                <button
                  type="button"
                  disabled={!bridgeAvailable || busy || !needsUnlock || pinEntry.length !== 4}
                  onClick={() => void unlockPin()}
                >
                  {busy ? strings.foflineActivatorUnlocking : strings.foflineActivatorUnlock}
                </button>
              )}
              {stage === "consume" && (
                <>
                  <button
                    type="button"
                    disabled={
                      !bridgeAvailable ||
                      busy ||
                      !pinUnlocked ||
                      status?.wall_clock_ok === false ||
                      airCode.trim().length === 0
                    }
                    onClick={() => void activateAir()}
                  >
                    {busy ? strings.foflineActivatorAiring : strings.foflineActivatorAir}
                  </button>
                  <button
                    type="button"
                    disabled={!bridgeAvailable || busy || !pinUnlocked}
                    onClick={() => void importCapsule()}
                  >
                    {busy ? strings.foflineActivatorImporting : strings.foflineActivatorImport}
                  </button>
                </>
              )}
              {stage === "portal" && (
                <button
                  type="button"
                  disabled={
                    !bridgeAvailable ||
                    busy ||
                    !pinUnlocked ||
                    !status?.channel_valid ||
                    Boolean(status?.channel_up)
                  }
                  onClick={() => void bringUpPortal()}
                >
                  {busy ? strings.foflineActivatorPortalUpping : strings.foflineActivatorPortalUp}
                </button>
              )}
              {stage === "spa" && (
                <button
                  type="button"
                  disabled={
                    !bridgeAvailable ||
                    busy ||
                    !pinUnlocked ||
                    !status?.allow_fofline ||
                    !status?.channel_up ||
                    status?.spa_state === "ready"
                  }
                  onClick={() => void syncSpa()}
                >
                  {busy ? strings.foflineActivatorSpaSyncing : strings.foflineActivatorSpaSync}
                </button>
              )}
              {stage === "login" && (
                <>
                  {status?.portal_hostmap !== "applied" && (
                    <button
                      type="button"
                      disabled={!bridgeAvailable || busy || !pinUnlocked || !status?.channel_up}
                      onClick={() => void repairHostmap()}
                    >
                      {busy
                        ? strings.foflineActivatorHostmapRepairing
                        : strings.foflineActivatorHostmapRepair}
                    </button>
                  )}
                  <button
                    type="button"
                    disabled={
                      !bridgeAvailable ||
                      busy ||
                      !pinUnlocked ||
                      !status?.allow_fofline ||
                      !status?.channel_up ||
                      status?.wall_clock_ok === false ||
                      status?.oidc_session === "present"
                    }
                    onClick={() => void startLogin()}
                  >
                    {busy ? strings.foflineActivatorOidcStarting : strings.foflineActivatorOidcStart}
                  </button>
                </>
              )}
              {stage === "ready" && (
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
                  onClick={() => void openProduct()}
                >
                  {busy ? strings.foflineActivatorProductOpening : strings.foflineActivatorProductOpen}
                </button>
              )}
              {status?.oidc_session === "present" && (
                <button
                  type="button"
                  disabled={!bridgeAvailable || busy || !pinUnlocked}
                  onClick={() => void endSession()}
                >
                  {busy ? strings.foflineActivatorOidcLeaving : strings.foflineActivatorOidcLeave}
                </button>
              )}
            </div>
          </div>
        ) : null}

        {tab === "status" ? (
          <div className="host-app-section">
            <div className="host-app-facts">
              <HostAppFact label={strings.foflineActivatorJourneyLabel} value={view.label} />
              <HostAppFact
                label={strings.foflineActivatorPin}
                value={
                  pinUnlocked
                    ? strings.foflineActivatorPinUnlocked
                    : pinLocked
                      ? strings.foflineActivatorPinLocked
                      : status?.pin_issued
                        ? strings.foflineActivatorPinIssued
                        : strings.foflineActivatorPinMissing
                }
              />
              <HostAppFact
                label={strings.foflineActivatorCapsule}
                value={
                  status?.capsule_imported
                    ? status.product_id ?? strings.yes
                    : strings.foflineActivatorCapsuleMissing
                }
              />
              <HostAppFact
                label={strings.foflineActivatorHostmap}
                value={
                  status?.portal_hostmap === "applied"
                    ? strings.foflineActivatorHostmapApplied
                    : strings.foflineActivatorHostmapMissing
                }
              />
              <HostAppFact
                label={strings.foflineActivatorSpa}
                value={
                  status?.spa_state === "ready"
                    ? strings.foflineActivatorSpaReady
                    : strings.foflineActivatorSpaMissing
                }
              />
              <HostAppFact
                label={strings.foflineActivatorOidcStart}
                value={
                  status?.oidc_session === "present"
                    ? strings.foflineActivatorOidcReady
                    : strings.foflineActivatorOidcMissing
                }
              />
            </div>
            <p className="secure-detail">{view.hint}</p>
            <div className="secure-actions">
              <button type="button" disabled={!bridgeAvailable || busy} onClick={() => void refresh()}>
                {strings.foflineActivatorRefresh}
              </button>
            </div>
          </div>
        ) : null}

        {tab === "lab" ? (
          <div className="host-app-section">
            <p className="fofline-shell-mark">
              {strings.foflineActivatorShellMark} {LAUNCHER_SHELL_BUILD}
            </p>
            <FoflineActivatorLab
              status={status}
              strings={strings}
              productAvailability={product?.availability ?? "disabled"}
              launcherId={launcherCap?.id}
              launcherReason={launcherCap?.reason}
              pinUnlocked={pinUnlocked}
              pinLocked={pinLocked}
              needsUnlock={needsUnlock}
              pinEntry={pinEntry}
              importPath={importPath}
              airCode={airCode}
              busy={busy}
              bridgeAvailable={bridgeAvailable}
              showConsumeFields={stage !== "consume"}
              onImportPath={setImportPath}
              onAirCode={setAirCode}
              onRefresh={() => void refresh()}
              onImport={() => void importCapsule()}
              onAir={() => void activateAir()}
              onSpa={() => void syncSpa()}
              onPortal={() => void bringUpPortal()}
              onLogin={() => void startLogin()}
              onLogout={() => void endSession()}
              onOpen={() => void openProduct()}
              onUnlock={() => void unlockPin()}
              windowMode={windowMode}
              onToggleWindow={() => void onToggleWindow()}
            />
          </div>
        ) : null}
      </HostAppShell>
    </div>
  );
}
