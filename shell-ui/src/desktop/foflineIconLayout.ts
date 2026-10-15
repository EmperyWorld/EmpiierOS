import {
  FOFLINE_HOST_MODULES,
  type HostModuleId,
} from "./foflineHostModules";

/** Harvested from EliteEW DESKTOP_GRID.GRID_SIZE — snap only, no DesktopIcon tree. */
export const HOST_ICON_GRID = 110;
const HOST_ICON_OFFSET = 16;
const STORAGE_KEY = "empiieros.fofline.host-icon-layout.v1";

export interface IconPoint {
  x: number;
  y: number;
}

export type HostIconLayout = Record<HostModuleId, IconPoint>;

export function defaultHostIconLayout(): HostIconLayout {
  const layout = {} as HostIconLayout;
  FOFLINE_HOST_MODULES.forEach((mod, index) => {
    layout[mod.id] = {
      x: HOST_ICON_OFFSET,
      y: HOST_ICON_OFFSET + index * HOST_ICON_GRID,
    };
  });
  return layout;
}

export function loadHostIconLayout(): HostIconLayout {
  const layout = defaultHostIconLayout();
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return layout;
    }
    const parsed = JSON.parse(raw) as Partial<Record<HostModuleId, IconPoint>>;
    for (const mod of FOFLINE_HOST_MODULES) {
      const point = parsed[mod.id];
      if (
        point &&
        Number.isFinite(point.x) &&
        Number.isFinite(point.y) &&
        point.x >= 0 &&
        point.y >= 0
      ) {
        layout[mod.id] = { x: point.x, y: point.y };
      }
    }
    return layout;
  } catch {
    return defaultHostIconLayout();
  }
}

export function saveHostIconLayout(layout: HostIconLayout): void {
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(layout));
}

export function clampHostIcon(x: number, y: number, maxX: number, maxY: number): IconPoint {
  return {
    x: Math.max(8, Math.min(maxX, x)),
    y: Math.max(8, Math.min(maxY, y)),
  };
}

export function snapHostIcon(x: number, y: number, maxX: number, maxY: number): IconPoint {
  const snappedX =
    Math.round((x - HOST_ICON_OFFSET) / HOST_ICON_GRID) * HOST_ICON_GRID + HOST_ICON_OFFSET;
  const snappedY =
    Math.round((y - HOST_ICON_OFFSET) / HOST_ICON_GRID) * HOST_ICON_GRID + HOST_ICON_OFFSET;
  return clampHostIcon(snappedX, snappedY, maxX, maxY);
}
