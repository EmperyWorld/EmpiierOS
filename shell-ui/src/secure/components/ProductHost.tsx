import { FoflineDesktop } from "../../desktop/FoflineDesktop";
import type { LocaleStrings } from "../locale";
import type { RuntimeEvent, SystemSnapshot } from "../types";
import type { HostWindowApp, Workspace } from "./Taskbar";

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

export function ProductHost({
  snapshot,
  events,
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
  return (
    <FoflineDesktop
      snapshot={snapshot}
      events={events}
      strings={strings}
      startOpen={startOpen}
      windowMode={windowMode}
      activatorOpen={activatorOpen}
      openApp={openApp}
      onCloseStart={onCloseStart}
      onSelectWorkspace={onSelectWorkspace}
      onActivatorOpenChange={onActivatorOpenChange}
      onOpenAppChange={onOpenAppChange}
      onToggleWindow={onToggleWindow}
      onApplied={onApplied}
      licensed={licensed}
      onOpenShield={onOpenShield}
    />
  );
}
