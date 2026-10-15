import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent, type ReactNode } from "react";
import { hostModuleTitle, type HostModuleId } from "../../desktop/foflineHostModules";
import { HelpTip } from "../design/controls";
import { hostWorkspaceHelp } from "../design/hostWorkspaceHelp";
import type { LocaleStrings } from "../locale";
import type { Workspace } from "./Taskbar";

interface Props {
  workspace: Exclude<Workspace, "eliteew">;
  strings: LocaleStrings;
  onClose: () => void;
  children: ReactNode;
}

type FramePos = { x: number; y: number };

const STORAGE_KEY = "empiieros.shell.host-workspace-pos.v1";
const PANEL_WIDTH = 820;
const PANEL_HEIGHT = 640;

function loadPos(id: string): FramePos | null {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as Partial<Record<string, FramePos>>;
    const pos = parsed[id];
    if (!pos || !Number.isFinite(pos.x) || !Number.isFinite(pos.y)) {
      return null;
    }
    return pos;
  } catch {
    return null;
  }
}

function savePos(id: string, pos: FramePos): void {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    const parsed = raw ? (JSON.parse(raw) as Record<string, FramePos>) : {};
    parsed[id] = pos;
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(parsed));
  } catch {
    /* seat storage is optional */
  }
}

function clampPos(x: number, y: number, stage: DOMRect, width: number, height: number): FramePos {
  const maxX = Math.max(8, stage.width - width - 8);
  const maxY = Math.max(8, stage.height - height - 8);
  return {
    x: Math.min(maxX, Math.max(8, x)),
    y: Math.min(maxY, Math.max(8, y)),
  };
}

function defaultPos(stage: DOMRect, width: number, height: number): FramePos {
  return clampPos(stage.width - width - 120, Math.max(24, (stage.height - height) / 2), stage, width, height);
}

export function WorkspaceFrame({ workspace, strings, onClose, children }: Props) {
  const frameRef = useRef<HTMLElement | null>(null);
  const dragRef = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    origX: number;
    origY: number;
  } | null>(null);
  const [pos, setPos] = useState<FramePos | null>(null);
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    const stage = document.getElementById("desktop-main")?.getBoundingClientRect();
    const frame = frameRef.current?.getBoundingClientRect();
    const width = frame?.width || Math.min(PANEL_WIDTH, (stage?.width ?? PANEL_WIDTH) - 48);
    const height = frame?.height || Math.min(PANEL_HEIGHT, (stage?.height ?? PANEL_HEIGHT) - 48);
    if (!stage) {
      setPos(loadPos(workspace) ?? { x: 24, y: 24 });
      return;
    }
    const stored = loadPos(workspace);
    setPos(stored ? clampPos(stored.x, stored.y, stage, width, height) : defaultPos(stage, width, height));
  }, [workspace]);

  const onBarPointerDown = (event: ReactPointerEvent<HTMLDivElement>): void => {
    if (event.button !== 0) {
      return;
    }
    if (event.target instanceof HTMLElement && event.target.closest("button")) {
      return;
    }
    if (!pos) {
      return;
    }
    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      origX: pos.x,
      origY: pos.y,
    };
    setDragging(true);
  };

  const onBarPointerMove = (event: ReactPointerEvent<HTMLDivElement>): void => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) {
      return;
    }
    const stage = document.getElementById("desktop-main")?.getBoundingClientRect();
    const frame = frameRef.current?.getBoundingClientRect();
    if (!stage || !frame) {
      return;
    }
    setPos(
      clampPos(
        drag.origX + event.clientX - drag.startX,
        drag.origY + event.clientY - drag.startY,
        stage,
        frame.width,
        frame.height,
      ),
    );
  };

  const endDrag = (event: ReactPointerEvent<HTMLDivElement>): void => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) {
      return;
    }
    dragRef.current = null;
    setDragging(false);
    if (pos) {
      savePos(workspace, pos);
    }
  };

  return (
    <article
      ref={frameRef}
      className={dragging ? "secure-app-window dragging" : "secure-app-window"}
      aria-labelledby="secure-app-window-title"
      style={pos ? { left: pos.x, top: pos.y } : undefined}
    >
      <div
        className="secure-app-window-bar"
        onPointerDown={onBarPointerDown}
        onPointerMove={onBarPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
      >
        <h2 id="secure-app-window-title" title={strings.workspaceDragHint}>
          {hostModuleTitle(strings, workspace as HostModuleId)}
        </h2>
        <div className="secure-app-window-bar-actions">
          <HelpTip label={strings.helpButtonLabel} text={hostWorkspaceHelp(strings, workspace)} />
          <button type="button" onClick={onClose}>
            {strings.foflineClose}
          </button>
        </div>
      </div>
      <div className="secure-app-window-body host-app-window-body">{children}</div>
    </article>
  );
}
