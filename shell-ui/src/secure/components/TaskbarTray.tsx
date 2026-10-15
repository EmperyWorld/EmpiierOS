import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import { hostModuleIconSrc } from "../../desktop/foflineHostModules";
import { DetailsSpoiler, TextField } from "../design/controls";
import { fill } from "../format";
import { type LocaleStrings } from "../locale";
import {
  dismissSecureNotification,
  listSecureNotifications,
  toggleQuietMode,
  type SecureNotificationItem,
} from "../notificationIpc";
import { runtimeRequest } from "../nativeTransport";
import type { SystemSnapshot } from "../types";
import {
  audioControlAvailable,
  clampSeatPercent,
  setSeatVolume,
  wheelSeatPercent,
} from "./taskbarTrayControl";
import {
  powerActionAvailable,
  trayBatteryTitle,
  trayClockLabel,
  trayLanguage,
  trayLanguageLabel,
  trayNetworkLabel,
  traySoundLabel,
  trayVolumePercent,
} from "./taskbarTrayModel";

type TrayPanel = "language" | "notifications" | "power" | "volume" | null;

interface Props {
  snapshot: SystemSnapshot | null;
  strings: LocaleStrings;
  language: string;
  onLanguageChange: (language: string) => void;
  onOpenNetwork: () => void;
  onOpenSystem: () => void;
  onOpenOta: () => void;
  otaOpen: boolean;
  licensed: boolean;
  onApplied: () => Promise<void>;
  notifyUnread: number;
  notifyCritical: number;
  onOpenShield: () => void;
}

function TrayIcon({
  src,
  label,
  hint,
  active,
  badgeCount,
  badgeCritical,
  buttonRef,
  onClick,
}: {
  src: string;
  label: string;
  hint: string;
  active?: boolean;
  badgeCount?: number;
  badgeCritical?: boolean;
  buttonRef?: RefObject<HTMLButtonElement | null>;
  onClick: () => void;
}) {
  const badge =
    badgeCount != null && badgeCount > 0
      ? badgeCount > 99
        ? "99+"
        : String(badgeCount)
      : null;
  return (
    <button
      type="button"
      ref={buttonRef}
      className={
        active
          ? "secure-taskbar-indicator secure-taskbar-icon active"
          : "secure-taskbar-indicator secure-taskbar-icon"
      }
      aria-label={
        badge ? `${label}. ${hint}. ${badgeCount} unread` : `${label}. ${hint}`
      }
      title={hint}
      onClick={onClick}
    >
      <img src={src} alt="" />
      {badge ? (
        <span
          className={
            badgeCritical
              ? "secure-taskbar-badge secure-taskbar-badge-critical"
              : "secure-taskbar-badge"
          }
          aria-hidden="true"
        >
          {badge}
        </span>
      ) : null}
      <span className="secure-taskbar-hint">{hint}</span>
    </button>
  );
}

export function TaskbarTray({
  snapshot,
  strings,
  language,
  onLanguageChange,
  onOpenNetwork,
  onOpenSystem,
  onOpenOta,
  otaOpen,
  licensed,
  onApplied,
  notifyUnread,
  notifyCritical,
  onOpenShield,
}: Props) {
  const rootRef = useRef<HTMLDivElement | null>(null);
  const volumeRef = useRef<HTMLButtonElement | null>(null);
  const [now, setNow] = useState(() => new Date());
  const [panel, setPanel] = useState<TrayPanel>(null);
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [volumeDraft, setVolumeDraft] = useState<number | null>(null);
  const [notifyItems, setNotifyItems] = useState<SecureNotificationItem[]>([]);
  const [notifyBusy, setNotifyBusy] = useState(false);
  const [notifyError, setNotifyError] = useState<string | null>(null);
  const [quietMode, setQuietMode] = useState(false);
  const canPower = powerActionAvailable(snapshot);
  const canVolume = audioControlAvailable(snapshot);
  const activeLanguage = trayLanguage(language);
  const observedVolume = trayVolumePercent(snapshot) ?? 50;
  const volume = volumeDraft ?? observedVolume;

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 15_000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (volumeDraft == null || snapshot?.audio.data.observed_percent == null) {
      return;
    }
    if (Math.abs(snapshot.audio.data.observed_percent - volumeDraft) <= 2) {
      setVolumeDraft(null);
    }
  }, [snapshot?.audio.data.observed_percent, volumeDraft]);

  useEffect(() => {
    if (!panel) {
      return;
    }
    const onPointer = (event: PointerEvent): void => {
      const root = rootRef.current;
      if (root && event.target instanceof Node && !root.contains(event.target)) {
        setPanel(null);
      }
    };
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape") {
        setPanel(null);
      }
    };
    window.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey);
    };
  }, [panel]);

  const reloadNotifications = useCallback(async (): Promise<void> => {
    try {
      const list = await listSecureNotifications();
      setNotifyItems(list.items);
      setQuietMode(list.quietMode);
      setNotifyError(null);
    } catch (error) {
      setNotifyError(error instanceof Error ? error.message : String(error));
    }
  }, []);

  useEffect(() => {
    if (panel !== "notifications") {
      return;
    }
    void reloadNotifications();
  }, [panel, reloadNotifications, notifyUnread]);

  const applyVolume = useCallback(
    async (next: number): Promise<void> => {
      if (!canVolume || busy) {
        return;
      }
      const percent = clampSeatPercent(next);
      setVolumeDraft(percent);
      setBusy(true);
      try {
        await setSeatVolume(percent);
        await onApplied();
      } finally {
        setBusy(false);
      }
    },
    [busy, canVolume, onApplied],
  );

  useEffect(() => {
    const node = volumeRef.current;
    if (!node) {
      return;
    }
    const onWheel = (event: globalThis.WheelEvent): void => {
      event.preventDefault();
      if (!canVolume || busy) {
        return;
      }
      const current = volumeDraft ?? trayVolumePercent(snapshot) ?? 50;
      void applyVolume(wheelSeatPercent(current, event.deltaY));
    };
    node.addEventListener("wheel", onWheel, { passive: false });
    return () => node.removeEventListener("wheel", onWheel);
  }, [applyVolume, busy, canVolume, snapshot, volumeDraft]);

  const runPower = async (action: "reboot" | "poweroff"): Promise<void> => {
    const expected = action === "reboot" ? "REBOOT" : "POWEROFF";
    if (confirm !== expected) {
      setMessage(strings.powerConfirmMismatch);
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      const response = await runtimeRequest(
        "system.power",
        { action, confirm: expected },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setConfirm("");
      setMessage(strings.powerScheduled);
      await onApplied();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const volumeHint = fill(strings.volumeNow, { percent: volume });

  const formatNotifyTime = (unix: number): string => {
    if (!unix) return "";
    try {
      return new Date(unix * 1000).toLocaleTimeString(language, {
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return "";
    }
  };

  const runNotifyAction = async (action: () => Promise<void>): Promise<void> => {
    if (notifyBusy) return;
    setNotifyBusy(true);
    try {
      await action();
      await reloadNotifications();
    } catch (error) {
      setNotifyError(error instanceof Error ? error.message : String(error));
    } finally {
      setNotifyBusy(false);
    }
  };

  const markNotificationRead = async (id: string): Promise<void> => {
    if (!id) return;
    await runNotifyAction(() => dismissSecureNotification(id, true));
  };

  const removeNotification = async (id: string): Promise<void> => {
    if (!id) return;
    await runNotifyAction(() => dismissSecureNotification(id, false));
  };

  const switchQuietMode = async (): Promise<void> => {
    await runNotifyAction(async () => {
      setQuietMode(await toggleQuietMode());
    });
  };

  return (
    <div className="secure-taskbar-tray" ref={rootRef} aria-label={strings.taskbarTray}>
      <TrayIcon
        src={hostModuleIconSrc("host-events.png")}
        label={strings.taskbarNotifications}
        hint={strings.taskbarNotifications}
        active={panel === "notifications"}
        badgeCount={notifyUnread}
        badgeCritical={notifyCritical > 0}
        onClick={() => setPanel((open) => (open === "notifications" ? null : "notifications"))}
      />
      {licensed ? (
        <TrayIcon
          src={hostModuleIconSrc("host-ota.png")}
          label={strings.workspaceOta}
          hint={strings.workspaceOta}
          active={otaOpen}
          onClick={onOpenOta}
        />
      ) : null}
      <TrayIcon
        src={hostModuleIconSrc("tray-network.png")}
        label={strings.workspaceNetwork}
        hint={trayNetworkLabel(snapshot, strings)}
        onClick={onOpenNetwork}
      />
      <TrayIcon
        src={hostModuleIconSrc("tray-battery.png")}
        label={strings.battery}
        hint={trayBatteryTitle(snapshot, strings)}
        onClick={onOpenSystem}
      />
      <TrayIcon
        src={hostModuleIconSrc("tray-volume.png")}
        label={strings.sound}
        hint={volumeHint}
        active={panel === "volume"}
        buttonRef={volumeRef}
        onClick={() => setPanel((open) => (open === "volume" ? null : "volume"))}
      />
      <button
        type="button"
        className="secure-taskbar-indicator secure-taskbar-clock"
        title={snapshot?.locale.data.timezone ?? strings.clock}
        onClick={onOpenSystem}
      >
        {trayClockLabel(now, language, snapshot?.locale.data.timezone ?? null)}
      </button>
      <button
        type="button"
        className={
          panel === "language"
            ? "secure-taskbar-indicator active"
            : "secure-taskbar-indicator"
        }
        aria-expanded={panel === "language"}
        aria-haspopup="menu"
        title={strings.taskbarLanguage}
        onClick={() => setPanel((open) => (open === "language" ? null : "language"))}
      >
        {trayLanguageLabel(language)}
      </button>
      <button
        type="button"
        className={
          panel === "power" ? "secure-taskbar-indicator active" : "secure-taskbar-indicator"
        }
        aria-expanded={panel === "power"}
        aria-haspopup="dialog"
        aria-label={strings.taskbarPower}
        title={strings.taskbarPower}
        onClick={() => {
          setMessage(null);
          setPanel((open) => (open === "power" ? null : "power"));
        }}
      >
        <span aria-hidden="true">⏻</span>
      </button>
      {panel === "volume" && (
        <div className="secure-taskbar-flyout" role="dialog" aria-label={strings.sound}>
          <p className="secure-detail">
            {traySoundLabel(snapshot, strings)} · {volumeHint}
          </p>
          <label className="ds-field">
            {strings.volume}
            <input
              type="range"
              min={10}
              max={100}
              step={5}
              value={volume}
              disabled={!canVolume || busy}
              onChange={(event) => void applyVolume(Number(event.target.value))}
              onWheel={(event) => {
                event.preventDefault();
                void applyVolume(wheelSeatPercent(volume, event.deltaY));
              }}
            />
          </label>
        </div>
      )}
      {panel === "notifications" && (
        <div
          className="secure-taskbar-flyout secure-taskbar-notify-flyout"
          role="dialog"
          aria-label={strings.taskbarNotifications}
        >
          <div className="secure-inline-actions secure-taskbar-quiet">
            <button
              type="button"
              aria-pressed={quietMode}
              className={quietMode ? "active" : undefined}
              disabled={notifyBusy}
              onClick={() => void switchQuietMode()}
            >
              {quietMode ? strings.taskbarQuietOn : strings.taskbarQuietOff}
            </button>
            <span className="secure-detail">{strings.taskbarQuietHint}</span>
          </div>
          {notifyError ? (
            <p className="secure-detail" role="alert">
              {strings.taskbarNotificationsFailed} {notifyError}
            </p>
          ) : null}
          {notifyItems.length === 0 ? (
            <p className="secure-detail">{strings.taskbarNotificationsEmpty}</p>
          ) : (
            <ul className="secure-taskbar-notify-list">
              {notifyItems.map((item) => (
                <li
                  key={item.id}
                  className={
                    item.read
                      ? "secure-taskbar-notify-item read"
                      : `secure-taskbar-notify-item severity-${item.severity}`
                  }
                >
                  <div className="secure-taskbar-notify-head">
                    <strong>{item.title}</strong>
                    <span className="secure-taskbar-notify-time">
                      {formatNotifyTime(item.createdAt)}
                    </span>
                  </div>
                  <p className="secure-detail">{item.message}</p>
                  <div className="secure-inline-actions">
                    {item.actionHint === "shield" ? (
                      <button
                        type="button"
                        disabled={notifyBusy}
                        onClick={() => {
                          setPanel(null);
                          onOpenShield();
                        }}
                      >
                        {strings.taskbarNotificationsOpenShield}
                      </button>
                    ) : null}
                    {!item.read ? (
                      <button
                        type="button"
                        disabled={notifyBusy}
                        onClick={() => void markNotificationRead(item.id)}
                      >
                        {strings.taskbarNotificationsMarkRead}
                      </button>
                    ) : null}
                    <button
                      type="button"
                      disabled={notifyBusy}
                      onClick={() => void removeNotification(item.id)}
                    >
                      {strings.taskbarNotificationsClear}
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      {panel === "language" && (
        <div className="secure-taskbar-flyout" role="menu" aria-label={strings.taskbarLanguage}>
          <button
            type="button"
            role="menuitem"
            className={activeLanguage === "ru" ? "active" : undefined}
            onClick={() => {
              onLanguageChange("ru");
              setPanel(null);
            }}
          >
            {strings.taskbarLanguageRu}
          </button>
          <button
            type="button"
            role="menuitem"
            className={activeLanguage === "en" ? "active" : undefined}
            onClick={() => {
              onLanguageChange("en");
              setPanel(null);
            }}
          >
            {strings.taskbarLanguageEn}
          </button>
        </div>
      )}
      {panel === "power" && (
        <div
          className="secure-taskbar-flyout secure-taskbar-power-flyout"
          role="dialog"
          aria-label={strings.taskbarPowerMenu}
        >
          <TextField
            label={strings.powerConfirmLabel}
            value={confirm}
            disabled={!canPower || busy}
            onChange={setConfirm}
          />
          <div className="secure-inline-actions">
            <button
              type="button"
              disabled={!canPower || busy}
              onClick={() => void runPower("reboot")}
            >
              {strings.reboot}
            </button>
            <button
              type="button"
              disabled={!canPower || busy}
              onClick={() => void runPower("poweroff")}
            >
              {strings.poweroff}
            </button>
          </div>
          {message && <p className="secure-detail">{message}</p>}
          <DetailsSpoiler summary={strings.moreDetails}>
            <p className="secure-detail">{strings.powerConfirmHint}</p>
          </DetailsSpoiler>
        </div>
      )}
    </div>
  );
}
