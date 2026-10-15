import { hostModuleIconSrc, hostModuleTitle } from "../../desktop/foflineHostModules";
import type { LocaleStrings } from "../locale";
import type { SystemSnapshot } from "../types";
import { TaskbarTray } from "./TaskbarTray";

export type Workspace =
  | "eliteew"
  | "system"
  | "network"
  | "diagnostics"
  | "shield"
  | "browser"
  | "ota";

export type HostWindowApp = "NotificationsApp" | "MusicPlayer";

export interface TaskbarTask {
  id: string;
  label: string;
  iconSrc: string;
  active: boolean;
}

interface Props {
  snapshot: SystemSnapshot | null;
  strings: LocaleStrings;
  language: string;
  workspace: Workspace;
  startOpen: boolean;
  tasks: TaskbarTask[];
  onSelectWorkspace: (workspace: Workspace) => void;
  onToggleStart: () => void;
  onSelectTask: (id: string) => void;
  onLanguageChange: (language: string) => void;
  onApplied: () => Promise<void>;
  licensed: boolean;
  notifyUnread: number;
  notifyCritical: number;
  onOpenShield: () => void;
}

export function Taskbar({
  snapshot,
  strings,
  language,
  workspace,
  startOpen,
  tasks,
  onSelectWorkspace,
  onToggleStart,
  onSelectTask,
  onLanguageChange,
  onApplied,
  licensed,
  notifyUnread,
  notifyCritical,
  onOpenShield,
}: Props) {
  return (
    <footer className="secure-taskbar" aria-label={strings.taskbarLabel}>
      <div className="secure-taskbar-left">
        <button
          type="button"
          className={
            workspace === "eliteew" && startOpen
              ? "secure-taskbar-start active"
              : "secure-taskbar-start"
          }
          data-start-menu-toggle="true"
          aria-label={strings.taskbarStart}
          aria-pressed={workspace === "eliteew" && startOpen}
          aria-expanded={startOpen}
          onClick={() => {
            if (workspace !== "eliteew") {
              onSelectWorkspace("eliteew");
            }
            onToggleStart();
          }}
        >
          <span className="secure-taskbar-logo">
            <img src={hostModuleIconSrc("emperyworld-icon.svg")} alt="" />
          </span>
        </button>
        <span className="secure-taskbar-divider" aria-hidden="true" />
      </div>
      <div className="secure-taskbar-windows" aria-label={strings.taskbarRunning}>
        {tasks.map((task) => (
          <button
            key={task.id}
            type="button"
            className={task.active ? "secure-taskbar-task active" : "secure-taskbar-task"}
            aria-pressed={task.active}
            onClick={() => onSelectTask(task.id)}
          >
            <img src={task.iconSrc} alt="" />
            <span>{task.label}</span>
          </button>
        ))}
      </div>
      <TaskbarTray
        snapshot={snapshot}
        strings={strings}
        language={language}
        onLanguageChange={onLanguageChange}
        onOpenNetwork={() => onSelectWorkspace("network")}
        onOpenSystem={() => onSelectWorkspace("system")}
        onOpenOta={() => onSelectWorkspace("ota")}
        otaOpen={workspace === "ota"}
        licensed={licensed}
        onApplied={onApplied}
        notifyUnread={notifyUnread}
        notifyCritical={notifyCritical}
        onOpenShield={onOpenShield}
      />
    </footer>
  );
}

export function hostWindowTask(
  strings: LocaleStrings,
  openApp: HostWindowApp | null,
  activatorOpen: boolean,
): TaskbarTask[] {
  const tasks: TaskbarTask[] = [];
  if (activatorOpen) {
    tasks.push({
      id: "launcher",
      label: hostModuleTitle(strings, "launcher"),
      iconSrc: hostModuleIconSrc("launcher.png"),
      active: true,
    });
  }
  if (openApp === "MusicPlayer") {
    tasks.push({
      id: "audio",
      label: hostModuleTitle(strings, "audio"),
      iconSrc: hostModuleIconSrc("host-audio.png"),
      active: true,
    });
  }
  if (openApp === "NotificationsApp") {
    tasks.push({
      id: "events",
      label: hostModuleTitle(strings, "events"),
      iconSrc: hostModuleIconSrc("host-events.png"),
      active: true,
    });
  }
  return tasks;
}
