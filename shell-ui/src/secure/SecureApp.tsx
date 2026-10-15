import { useCallback, useEffect, useMemo, useState } from "react";
import { DiagnosticsPanel } from "./components/DiagnosticsPanel";
import { NetworkPanel } from "./components/NetworkPanel";
import { ProductHost } from "./components/ProductHost";
import { SeatBrowserCard } from "./components/SeatBrowserCard";
import { OtaPanel } from "./components/OtaPanel";
import { SeatShieldCard } from "./components/SeatShieldCard";
import { SystemPanel } from "./components/SystemPanel";
import {
  hostWindowTask,
  Taskbar,
  type HostWindowApp,
  type TaskbarTask,
  type Workspace,
} from "./components/Taskbar";
import { DetailsSpoiler } from "./design/controls";
import { WorkspaceFrame } from "./components/WorkspaceFrame";
import { hostModuleIconSrc, hostModuleTitle } from "../desktop/foflineHostModules";
import {
  seatLicenseReady,
  type LauncherStageInput,
} from "../desktop/foflineLauncherStage";
import { localizedValue, localeStrings } from "./locale";
import { nativeBridgeAvailable, runtimeRequest } from "./nativeTransport";
import { NotificationToastStack } from "./components/NotificationToastStack";
import type { SecureNotificationItem } from "./notificationIpc";
import { useSeatShieldNotifications } from "./useSeatShieldNotifications";
import { useSystemSnapshot } from "./useSystemSnapshot";
import { installViewportLockedWheelScroll } from "./wheelScrollTarget";
import "./secure.css";
import "./design/controls.css";

const WORKSPACES: Workspace[] = [
  "eliteew",
  "system",
  "network",
  "diagnostics",
  "shield",
  "browser",
  "ota",
];
const UI_LANGUAGE_KEY = "empiieros.shell.ui-language.v1";

function storedUiLanguage(): string | null {
  try {
    const stored = window.localStorage.getItem(UI_LANGUAGE_KEY);
    return stored === "ru" || stored === "en" ? stored : null;
  } catch {
    return null;
  }
}

function initialLanguage(): string {
  return (
    (import.meta.env.VITE_UI_LOCALE as string | undefined) ||
    storedUiLanguage() ||
    navigator.language ||
    "ru"
  );
}

function persistUiLanguage(language: string): void {
  try {
    window.localStorage.setItem(
      UI_LANGUAGE_KEY,
      language.toLocaleLowerCase().startsWith("ru") ? "ru" : "en",
    );
  } catch {
    /* seat storage is optional */
  }
}

export default function SecureApp() {
  const { snapshot, events, loading, error, refresh } = useSystemSnapshot();
  const [language, setLanguage] = useState(initialLanguage);
  const strings = useMemo(() => localeStrings(language), [language]);
  const [workspace, setWorkspace] = useState<Workspace>("eliteew");
  const [startOpen, setStartOpen] = useState(false);
  const [activatorOpen, setActivatorOpen] = useState(false);
  const [openApp, setOpenApp] = useState<HostWindowApp | null>(null);
  const [windowMode, setWindowMode] = useState<"kiosk" | "windowed">("kiosk");
  const [windowError, setWindowError] = useState<string | null>(null);
  const [windowTechnical, setWindowTechnical] = useState<string | null>(null);
  const [licensed, setLicensed] = useState(false);
  const [notifyUnread, setNotifyUnread] = useState(0);
  const [notifyCritical, setNotifyCritical] = useState(0);
  const [notifyItems, setNotifyItems] = useState<SecureNotificationItem[]>([]);
  const bridgeAvailable = nativeBridgeAvailable();
  const onNotifyUnread = useCallback((unread: number, critical: number) => {
    setNotifyUnread(unread);
    setNotifyCritical(critical);
  }, []);
  const onNotifyList = useCallback((items: SecureNotificationItem[]) => {
    setNotifyItems(items);
  }, []);
  useSeatShieldNotifications(strings, onNotifyUnread, onNotifyList);

  useEffect(() => {
    /* Always install: native GTK must not own wheel (return FALSE). Document
     * is overflow:hidden; host panels need a DOM scrollport walker. */
    return installViewportLockedWheelScroll();
  }, []);

  useEffect(() => {
    if (import.meta.env.VITE_UI_LOCALE || storedUiLanguage()) {
      return;
    }
    const observed = snapshot?.locale.data.locale;
    if (observed) {
      setLanguage(observed);
    }
  }, [snapshot?.locale.data.locale]);

  const selectLanguage = (next: string): void => {
    persistUiLanguage(next);
    setLanguage(next);
  };

  useEffect(() => {
    const onMode = (event: Event): void => {
      const detail = (event as CustomEvent<{ mode?: string }>).detail;
      if (detail?.mode === "kiosk" || detail?.mode === "windowed") {
        setWindowMode(detail.mode);
      }
    };
    window.addEventListener("empiieros-window-mode", onMode);
    return () => window.removeEventListener("empiieros-window-mode", onMode);
  }, []);

  useEffect(() => {
    let cancelled = false;
    const tick = async (): Promise<void> => {
      try {
        const response = await runtimeRequest<LauncherStageInput>("launcher.status", {}, 5_000);
        if (!cancelled) {
          setLicensed(response.ok ? seatLicenseReady(response.data) : false);
        }
      } catch {
        if (!cancelled) {
          setLicensed(false);
        }
      }
    };
    void tick();
    const timer = window.setInterval(() => {
      void tick();
    }, 4000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    if (!licensed) {
      setWorkspace("eliteew");
      setStartOpen(false);
      setOpenApp(null);
      setActivatorOpen(true);
    }
  }, [licensed]);

  useEffect(() => {
    const onHost = (event: Event): void => {
      const detail = (event as CustomEvent<{ workspace?: string; activator?: boolean }>)
        .detail;
      const next = detail?.workspace;
      if (
        next !== "eliteew" &&
        next !== "system" &&
        next !== "network" &&
        next !== "diagnostics" &&
        next !== "shield" &&
        next !== "browser" &&
        next !== "ota"
      ) {
        return;
      }
      if (!licensed && next !== "eliteew" && next !== "network") {
        setStartOpen(false);
        setOpenApp(null);
        setWorkspace("eliteew");
        setActivatorOpen(true);
        return;
      }
      setStartOpen(false);
      setWorkspace(next);
      if (next !== "eliteew") {
        setActivatorOpen(false);
        setOpenApp(null);
        return;
      }
      if (detail?.activator) {
        setOpenApp(null);
        setActivatorOpen(true);
      }
    };
    window.addEventListener("empiieros-host-workspace", onHost);
    return () => window.removeEventListener("empiieros-host-workspace", onHost);
  }, [licensed]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      const target = event.target;
      if (
        target instanceof HTMLElement &&
        (target.tagName === "INPUT" ||
          target.tagName === "SELECT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable)
      ) {
        return;
      }
      if (!licensed) {
        if (event.key === "1") {
          setWorkspace("eliteew");
          setActivatorOpen(true);
        } else if (event.key === "3") {
          setStartOpen(false);
          setOpenApp(null);
          setActivatorOpen(false);
          setWorkspace("network");
        }
        return;
      }
      if (event.key === "1") {
        setWorkspace("eliteew");
      } else if (event.key === "2") {
        setStartOpen(false);
        setWorkspace("system");
      } else if (event.key === "3") {
        setStartOpen(false);
        setWorkspace("network");
      } else if (event.key === "4") {
        setStartOpen(false);
        setWorkspace("diagnostics");
      } else if (event.key === "5") {
        setStartOpen(false);
        setWorkspace("shield");
      } else if (event.key === "6") {
        setStartOpen(false);
        setWorkspace("browser");
      } else if (event.key === "7") {
        setStartOpen(false);
        setWorkspace("ota");
      } else if (event.altKey && (event.key === "ArrowLeft" || event.key === "ArrowRight")) {
        event.preventDefault();
        const index = WORKSPACES.indexOf(workspace);
        const next =
          event.key === "ArrowRight"
            ? WORKSPACES[(index + 1) % WORKSPACES.length]
            : WORKSPACES[(index - 1 + WORKSPACES.length) % WORKSPACES.length];
        if (next !== "eliteew") {
          setStartOpen(false);
        }
        setWorkspace(next);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [licensed, workspace]);

  const openShield = useCallback((): void => {
    if (!licensed) {
      setStartOpen(false);
      setOpenApp(null);
      setWorkspace("eliteew");
      setActivatorOpen(true);
      return;
    }
    setStartOpen(false);
    setOpenApp(null);
    setActivatorOpen(false);
    setWorkspace("shield");
  }, [licensed]);

  const selectWorkspace = (next: Workspace): void => {
    if (!licensed && next !== "eliteew" && next !== "network") {
      setStartOpen(false);
      setOpenApp(null);
      setWorkspace("eliteew");
      setActivatorOpen(true);
      return;
    }
    setStartOpen(false);
    if (next !== "eliteew") {
      setActivatorOpen(false);
      setOpenApp(null);
    }
    setWorkspace(next);
  };

  const selectTask = (id: string): void => {
    if (!licensed && id !== "launcher" && id !== "network") {
      setWorkspace("eliteew");
      setStartOpen(false);
      setOpenApp(null);
      setActivatorOpen(true);
      return;
    }
    if (id === "launcher") {
      if (licensed && workspace === "eliteew" && activatorOpen) {
        setActivatorOpen(false);
        return;
      }
      setWorkspace("eliteew");
      setStartOpen(false);
      setOpenApp(null);
      setActivatorOpen(true);
      return;
    }
    if (id === "audio") {
      setWorkspace("eliteew");
      setStartOpen(false);
      setActivatorOpen(false);
      setOpenApp("MusicPlayer");
      return;
    }
    if (id === "events") {
      setWorkspace("eliteew");
      setStartOpen(false);
      setActivatorOpen(false);
      setOpenApp("NotificationsApp");
      return;
    }
    if (
      id === "system" ||
      id === "network" ||
      id === "diagnostics" ||
      id === "shield" ||
      id === "browser" ||
      id === "ota"
    ) {
      selectWorkspace(id);
    }
  };

  const runningTasks = useMemo((): TaskbarTask[] => {
    const tasks = hostWindowTask(strings, openApp, activatorOpen);
    if (workspace !== "eliteew") {
      tasks.push({
        id: workspace,
        label: hostModuleTitle(strings, workspace),
        iconSrc: hostModuleIconSrc(
          workspace === "shield"
            ? "host-shield.png"
            : workspace === "browser"
              ? "host-browser.png"
              : workspace === "system"
                ? "host-system.png"
                : workspace === "network"
                  ? "host-network.png"
                  : workspace === "ota"
                    ? "host-ota.png"
                    : "host-diag.png",
        ),
        active: true,
      });
    }
    return tasks;
  }, [strings, openApp, activatorOpen, workspace]);

  const toggleWindow = async (): Promise<void> => {
    const next = windowMode === "kiosk" ? "windowed" : "kiosk";
    setWindowError(null);
    setWindowTechnical(null);
    try {
      const response = await runtimeRequest<{ mode: "kiosk" | "windowed" }>(
        "shell.window_mode",
        { mode: next },
        5_000,
      );
      if (!response.ok || response.data?.mode !== next) {
        throw new Error(response.error?.code ?? response.status);
      }
      setWindowMode(next);
    } catch (caught) {
      const code = caught instanceof Error ? caught.message : String(caught);
      if (code === "window_mode_service_disabled") {
        setWindowError(strings.windowModeServiceDisabled);
        setWindowTechnical(null);
      } else {
        setWindowError(strings.windowModeFailed);
        setWindowTechnical(localizedValue(strings, code));
      }
    }
  };

  const shellClass = [
    "secure-shell",
    "secure-shell-desktop",
    windowMode === "windowed" ? "secure-shell-windowed" : "",
    "secure-shell-fofline",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <main
      className={shellClass}
      data-overall={snapshot?.overall_status ?? "unknown"}
      data-workspace={workspace}
    >
      <section className="secure-desktop-stage" id="desktop-main">
        <ProductHost
          snapshot={snapshot}
          events={events}
          strings={strings}
          startOpen={startOpen}
          windowMode={windowMode}
          activatorOpen={activatorOpen}
          openApp={openApp}
          onCloseStart={() => setStartOpen(false)}
          onSelectWorkspace={selectWorkspace}
          onActivatorOpenChange={setActivatorOpen}
          onOpenAppChange={setOpenApp}
          onToggleWindow={toggleWindow}
          onApplied={refresh}
          licensed={licensed}
          onOpenShield={openShield}
        />
        <NotificationToastStack
          strings={strings}
          incoming={notifyItems}
          onOpenShield={openShield}
        />
        {loading && !snapshot && <p className="secure-loading">{strings.refreshing}</p>}
        {error && (
          <section className="secure-error" role="alert">
            <h2>{strings.runtimeUnavailable}</h2>
            <DetailsSpoiler summary={strings.moreDetails}>
              <p>{error}</p>
            </DetailsSpoiler>
            <button onClick={() => void refresh()}>{strings.retry}</button>
          </section>
        )}
        {windowError && (
          <div className="secure-error" role="alert">
            <p>
              {windowError}
              {!bridgeAvailable ? ` · ${strings.nativeBridgeUnavailable}` : ""}
            </p>
            {windowTechnical ? (
              <DetailsSpoiler summary={strings.moreDetails}>
                <span>{windowTechnical}</span>
              </DetailsSpoiler>
            ) : null}
          </div>
        )}
        {workspace !== "eliteew" && (
          <WorkspaceFrame
            workspace={workspace}
            strings={strings}
            onClose={() => selectWorkspace("eliteew")}
          >
            {snapshot && workspace === "system" && (
              <SystemPanel snapshot={snapshot} strings={strings} onApplied={refresh} />
            )}
            {snapshot && workspace === "network" && (
              <NetworkPanel snapshot={snapshot} strings={strings} onApplied={refresh} />
            )}
            {snapshot && workspace === "diagnostics" && (
              <DiagnosticsPanel snapshot={snapshot} events={events} strings={strings} />
            )}
            {workspace === "shield" && <SeatShieldCard strings={strings} />}
            {workspace === "browser" && <SeatBrowserCard strings={strings} />}
            {workspace === "ota" && <OtaPanel strings={strings} />}
          </WorkspaceFrame>
        )}
      </section>

      <Taskbar
        snapshot={snapshot}
        strings={strings}
        language={language}
        workspace={workspace}
        startOpen={startOpen}
        tasks={runningTasks}
        onSelectWorkspace={selectWorkspace}
        onToggleStart={() => setStartOpen((open) => !open)}
        onSelectTask={selectTask}
        onLanguageChange={selectLanguage}
        onApplied={refresh}
        licensed={licensed}
        notifyUnread={notifyUnread}
        notifyCritical={notifyCritical}
        onOpenShield={openShield}
      />
    </main>
  );
}
