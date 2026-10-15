import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import type { HostWindowApp, Workspace } from "../secure/components/Taskbar";
import { DetailsSpoiler, HelpTip, SelectField, StatusPill } from "../secure/design/controls";
import { localizedValue, type LocaleStrings } from "../secure/locale";
import { runtimeRequest } from "../secure/nativeTransport";
import { SecureNotificationList } from "../secure/components/SecureNotificationList";
import type { RuntimeEvent, SystemSnapshot } from "../secure/types";
import { FoflineActivator } from "./FoflineActivator";
import { FOFLINE_CATALOG, foflineHostTiles, type FoflineApp } from "./foflineCatalog";
import {
  hostModuleIconSrc,
  hostModuleTitle,
  isSeatFloorModule,
  visibleHostModules,
  type HostModule,
  type HostModuleId,
} from "./foflineHostModules";
import {
  clampHostIcon,
  loadHostIconLayout,
  saveHostIconLayout,
  snapHostIcon,
  type HostIconLayout,
} from "./foflineIconLayout";
import { HostAppFact, HostAppShell } from "../secure/design/HostAppShell";
import "./fofline.css";

interface Props {
  snapshot: SystemSnapshot | null;
  events: RuntimeEvent[];
  strings: LocaleStrings;
  startOpen: boolean;
  windowMode: "kiosk" | "windowed";
  activatorOpen: boolean;
  openApp: HostWindowApp | null;
  onCloseStart: () => void;
  onSelectWorkspace: (workspace: Workspace) => void;
  onActivatorOpenChange: (open: boolean) => void;
  onOpenAppChange: (app: HostWindowApp | null) => void;
  onToggleWindow: () => Promise<void>;
  onApplied: () => Promise<void>;
  licensed: boolean;
  onOpenShield: () => void;
}

export function FoflineDesktop({
  snapshot,
  events: _events,
  strings,
  startOpen,
  windowMode,
  activatorOpen,
  openApp,
  onCloseStart,
  onSelectWorkspace,
  onActivatorOpenChange,
  onOpenAppChange,
  onToggleWindow,
  onApplied,
  licensed,
  onOpenShield,
}: Props) {
  const hostTiles = foflineHostTiles();
  const selected = hostTiles.find((item) => item.appComponent === openApp) ?? null;
  const [layout, setLayout] = useState<HostIconLayout>(loadHostIconLayout);
  const [draggingId, setDraggingId] = useState<HostModuleId | null>(null);
  const dragRef = useRef<{
    id: HostModuleId;
    pointerId: number;
    startX: number;
    startY: number;
    origX: number;
    origY: number;
    moved: boolean;
  } | null>(null);

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape") {
        if (!licensed) {
          onOpenAppChange(null);
          onActivatorOpenChange(true);
          return;
        }
        if (openApp) {
          onOpenAppChange(null);
        } else if (activatorOpen) {
          onActivatorOpenChange(false);
        } else {
          onCloseStart();
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [licensed, openApp, activatorOpen, onCloseStart, onOpenAppChange, onActivatorOpenChange]);

  const openActivator = (): void => {
    onOpenAppChange(null);
    onActivatorOpenChange(true);
    onCloseStart();
  };

  useEffect(() => {
    if (!licensed) {
      onOpenAppChange(null);
      onActivatorOpenChange(true);
    }
  }, [licensed, onActivatorOpenChange, onOpenAppChange]);

  const openModule = (mod: HostModule): void => {
    if (!licensed && !isSeatFloorModule(mod.id)) {
      openActivator();
      return;
    }
    if (mod.opens === "workspace" && mod.workspace) {
      onSelectWorkspace(mod.workspace);
      return;
    }
    if (mod.id === "launcher") {
      openActivator();
      return;
    }
    if (mod.windowApp) {
      onActivatorOpenChange(false);
      onOpenAppChange(mod.windowApp);
      onCloseStart();
    }
  };

  const iconBounds = (canvas: Element | null): { maxX: number; maxY: number } => {
    const box = canvas instanceof HTMLElement ? canvas : null;
    return {
      maxX: Math.max(8, (box?.clientWidth ?? 800) - 120),
      maxY: Math.max(8, (box?.clientHeight ?? 600) - 120),
    };
  };

  const onIconPointerDown = (id: HostModuleId, event: ReactPointerEvent<HTMLButtonElement>): void => {
    if (event.button !== 0) {
      return;
    }
    event.stopPropagation();
    const point = layout[id];
    dragRef.current = {
      id,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      origX: point.x,
      origY: point.y,
      moved: false,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const onIconPointerMove = (event: ReactPointerEvent<HTMLButtonElement>): void => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) {
      return;
    }
    const dx = event.clientX - drag.startX;
    const dy = event.clientY - drag.startY;
    if (!drag.moved && dx * dx + dy * dy < 36) {
      return;
    }
    drag.moved = true;
    setDraggingId(drag.id);
    const { maxX, maxY } = iconBounds(event.currentTarget.closest(".fofline-canvas"));
    setLayout((prev) => ({
      ...prev,
      [drag.id]: clampHostIcon(drag.origX + dx, drag.origY + dy, maxX, maxY),
    }));
  };

  const onIconPointerUp = (mod: HostModule, event: ReactPointerEvent<HTMLButtonElement>): void => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) {
      return;
    }
    dragRef.current = null;
    setDraggingId(null);
    if (drag.moved) {
      const { maxX, maxY } = iconBounds(event.currentTarget.closest(".fofline-canvas"));
      setLayout((prev) => {
        const next = {
          ...prev,
          [drag.id]: snapHostIcon(prev[drag.id].x, prev[drag.id].y, maxX, maxY),
        };
        saveHostIconLayout(next);
        return next;
      });
      return;
    }
    openModule(mod);
  };

  return (
    <div className="fofline-desktop">
      <div
        className="fofline-wallpaper"
        aria-hidden="true"
        style={{
          backgroundImage: `linear-gradient(180deg, rgb(3 7 18 / 0.28), rgb(3 7 18 / 0.5)), url("${hostModuleIconSrc("host-wallpaper.jpg")}")`,
        }}
      />
      <div
        className="fofline-canvas"
        onClick={() => {
          if (licensed) {
            onCloseStart();
          }
        }}
      >
        <section className="fofline-icons" aria-label={strings.foflineHostTilesTitle}>
          {visibleHostModules(licensed).map((mod) => (
            <button
              key={mod.id}
              type="button"
              className={iconClassName(mod, openApp, activatorOpen, draggingId)}
              style={{ left: layout[mod.id].x, top: layout[mod.id].y }}
              onPointerDown={(event) => onIconPointerDown(mod.id, event)}
              onPointerMove={onIconPointerMove}
              onPointerUp={(event) => onIconPointerUp(mod, event)}
              onPointerCancel={(event) => onIconPointerUp(mod, event)}
              onClick={(event) => event.stopPropagation()}
            >
              <span className="fofline-glyph" aria-hidden="true">
                <img src={hostModuleIconSrc(mod.icon)} alt="" draggable={false} />
              </span>
              <strong>{hostModuleTitle(strings, mod.id)}</strong>
            </button>
          ))}
        </section>
        {activatorOpen && (
          <article
            className="fofline-window"
            onClick={(event) => event.stopPropagation()}
            aria-labelledby="fofline-activator-title"
          >
            <div className="fofline-window-bar">
              <div>
                <h2 id="fofline-activator-title">{strings.foflineActivatorTitle}</h2>
                {!licensed && <p>{strings.seatLicenseOnly}</p>}
              </div>
              <div className="secure-app-window-bar-actions">
                <HelpTip label={strings.helpButtonLabel} text={strings.hostHelpLauncher} />
                {licensed ? (
                  <button type="button" onClick={() => onActivatorOpenChange(false)}>
                    {strings.foflineClose}
                  </button>
                ) : null}
              </div>
            </div>
            <FoflineActivator
              snapshot={snapshot}
              strings={strings}
              windowMode={windowMode}
              onToggleWindow={onToggleWindow}
            />
          </article>
        )}
        {selected && (
          <article
            className="fofline-window"
            onClick={(event) => event.stopPropagation()}
            aria-labelledby="fofline-window-title"
          >
            <div className="fofline-window-bar">
              <div>
                <h2 id="fofline-window-title">{appCopy(strings, selected).name}</h2>
              </div>
              <button type="button" onClick={() => onOpenAppChange(null)}>
                {strings.foflineClose}
              </button>
            </div>
            <FoflineAppBody
              app={selected}
              snapshot={snapshot}
              strings={strings}
              onApplied={onApplied}
              onOpenShield={onOpenShield}
            />
          </article>
        )}
      </div>
      {startOpen && (
        <aside
          className="fofline-start"
          aria-label={strings.foflineStartMenu}
          onClick={(event) => event.stopPropagation()}
        >
          <div className="fofline-start-head">
            <h2>{strings.appTitle}</h2>
            <HelpTip label={strings.helpButtonLabel} text={strings.hostHelpStartMenu} />
          </div>
          <ul className="fofline-start-list">
            {visibleHostModules(licensed).map((mod) => (
              <li key={mod.id}>
                <button type="button" onClick={() => openModule(mod)}>
                  <span>{hostModuleTitle(strings, mod.id)}</span>
                  <small>{moduleKind(strings, mod)}</small>
                </button>
              </li>
            ))}
          </ul>
          <DetailsSpoiler summary={strings.foflinePackInside}>
            <ul className="fofline-store">
              {FOFLINE_CATALOG.filter((app) => app.localKind === "pack").map((app) => (
                <li key={app.id}>
                  <strong>{appCopy(strings, app).name}</strong>
                  <div>{appCopy(strings, app).description}</div>
                </li>
              ))}
            </ul>
          </DetailsSpoiler>
        </aside>
      )}
    </div>
  );
}

function iconClassName(
  mod: HostModule,
  openApp: HostWindowApp | null,
  activatorOpen: boolean,
  draggingId: HostModuleId | null,
): string {
  const classes = ["fofline-icon"];
  if (mod.id === "launcher" ? activatorOpen : Boolean(mod.windowApp && openApp === mod.windowApp)) {
    classes.push("active");
  }
  if (draggingId === mod.id) {
    classes.push("dragging");
  }
  return classes.join(" ");
}

function moduleKind(strings: LocaleStrings, mod: HostModule): string {
  if (mod.opens === "workspace") return strings.foflineHostOsKind;
  if (mod.id === "launcher") return strings.foflineHostLauncherKind;
  return strings.foflineHostLocalKind;
}

function FoflineAppBody({
  app,
  snapshot,
  strings,
  onApplied,
  onOpenShield,
}: {
  app: FoflineApp;
  snapshot: SystemSnapshot | null;
  strings: LocaleStrings;
  onApplied: () => Promise<void>;
  onOpenShield: () => void;
}) {
  if (app.localKind === "events") {
    return <EventsBody strings={strings} onOpenShield={onOpenShield} />;
  }
  if (app.localKind === "audio") {
    return <AudioBody snapshot={snapshot} strings={strings} onApplied={onApplied} />;
  }
  if (app.localKind === "about") {
    return <AboutBody snapshot={snapshot} strings={strings} />;
  }
  return (
    <div>
      <StatusPill value={strings.foflinePackMissing} state="disabled" />
      <p>{strings.foflinePackMissingLead}</p>
    </div>
  );
}

function EventsBody({
  strings,
  onOpenShield,
}: {
  strings: LocaleStrings;
  onOpenShield: () => void;
}) {
  const [total, setTotal] = useState(0);
  const [unread, setUnread] = useState(0);

  return (
    <HostAppShell
      ariaLabel={strings.foflineLocalEvents}
      className="fofline-host-window-shell"
      tabs={[
        {
          id: "overview",
          label: strings.hostAppTabOverview,
          count: unread > 0 ? unread : total > 0 ? total : undefined,
        },
      ]}
      activeTab="overview"
      onTabChange={() => undefined}
      statusChips={[
        {
          label:
            unread > 0
              ? `${strings.taskbarNotifications} · ${unread}`
              : total > 0
                ? strings.taskbarNotifications
                : strings.taskbarNotificationsEmpty,
          state: unread > 0 ? "local" : total > 0 ? "connected" : "connected",
        },
      ]}
      facts={
        <HostAppFact
          label={strings.taskbarNotifications}
          value={unread > 0 ? `${unread} / ${total}` : String(total)}
        />
      }
    >
      <SecureNotificationList
        strings={strings}
        onOpenShield={() => {
          onOpenShield();
        }}
        onItemsChange={(items, unreadCount) => {
          setTotal(items.length);
          setUnread(unreadCount);
        }}
      />
    </HostAppShell>
  );
}

function AudioBody({
  snapshot,
  strings,
  onApplied,
}: {
  snapshot: SystemSnapshot | null;
  strings: LocaleStrings;
  onApplied: () => Promise<void>;
}) {
  const available = Boolean(
    snapshot?.capabilities.some(
      (item) => item.id === "audio.control" && item.availability === "available",
    ),
  );
  const observed = snapshot?.audio.data.observed_percent;
  const [percent, setPercent] = useState(String(observed ?? 50));
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [technical, setTechnical] = useState<string | null>(null);
  const options = [10, 25, 50, 75, 100].map((value) => ({
    value: String(value),
    label: `${value}%`,
  }));

  const apply = async (): Promise<void> => {
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "audio.control",
        {
          action: "set_volume",
          percent: Number.parseInt(percent, 10),
          name: "Speaker",
        },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.volumeApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <HostAppShell
      ariaLabel={strings.foflineLocalAudio}
      className="fofline-host-window-shell"
      tabs={[{ id: "overview", label: strings.hostAppTabOverview }]}
      activeTab="overview"
      onTabChange={() => undefined}
      busy={busy}
      message={message}
      statusChips={[
        {
          label: available ? strings.ready : strings.unavailable,
          state: available ? "connected" : "unavailable",
        },
        {
          label: observed != null ? `${observed}%` : strings.sound,
          state: observed != null ? "local" : "unavailable",
        },
      ]}
      facts={
        <>
          <HostAppFact
            label={strings.sound}
            value={snapshot?.audio.data.playback_name ?? strings.unknown}
          />
          <HostAppFact
            label={strings.volume}
            value={observed != null ? `${observed}%` : strings.unknown}
          />
        </>
      }
    >
      <div className="host-app-section">
        <SelectField
          label={strings.volume}
          value={percent}
          disabled={!available || busy}
          options={options}
          onChange={setPercent}
        />
        <button type="button" disabled={!available || busy} onClick={() => void apply()}>
          {busy ? strings.applying : strings.applyVolume}
        </button>
        <DetailsSpoiler summary={strings.moreDetails}>
          <p>{strings.volumeHint}</p>
          {technical ? <p className="secure-detail">{technical}</p> : null}
        </DetailsSpoiler>
      </div>
    </HostAppShell>
  );
}

function AboutBody({
  snapshot,
  strings,
}: {
  snapshot: SystemSnapshot | null;
  strings: LocaleStrings;
}) {
  const product = snapshot?.capabilities.find((item) => item.id === "product.open");

  return (
    <HostAppShell
      ariaLabel={strings.foflineLocalAbout}
      className="fofline-host-window-shell"
      statusChips={[
        {
          label: localizedValue(strings, product?.availability ?? "disabled"),
          state: product?.availability ?? "disabled",
        },
      ]}
      facts={
        <>
          <HostAppFact
            label={strings.computerName}
            value={snapshot?.host.hostname ?? strings.unknown}
          />
          <HostAppFact
            label={strings.locale}
            value={snapshot?.locale.data.locale ?? strings.unknown}
          />
        </>
      }
    >
      <div className="host-app-section">
        <p className="secure-detail">{strings.foflineAboutBody}</p>
        <p className="secure-detail">{strings.foflineAboutRights}</p>
      </div>
    </HostAppShell>
  );
}

function appCopy(strings: LocaleStrings, app: FoflineApp): { name: string; description: string } {
  const apps = strings.foflineApps as Record<string, { name: string; description: string }>;
  return apps[app.appComponent] ?? { name: app.appComponent, description: app.appComponent };
}
