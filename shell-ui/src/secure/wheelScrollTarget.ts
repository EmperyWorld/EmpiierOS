/** Nearest scrollport under pointer (viewport-locked shell has no document scroll). */
function overflowScrollableY(el: Element): boolean {
  const oy = getComputedStyle(el).overflowY;
  if (oy !== "auto" && oy !== "scroll" && oy !== "overlay") {
    return false;
  }
  return el.scrollHeight > el.clientHeight + 1;
}

function overflowScrollableX(el: Element): boolean {
  const ox = getComputedStyle(el).overflowX;
  if (ox !== "auto" && ox !== "scroll" && ox !== "overlay") {
    return false;
  }
  return el.scrollWidth > el.clientWidth + 1;
}

function wheelDeltaPixels(event: WheelEvent): { dx: number; dy: number } {
  let dx = event.deltaX;
  let dy = event.deltaY;
  if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
    dx *= 16;
    dy *= 16;
  } else if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
    dx *= window.innerWidth;
    dy *= window.innerHeight;
  }
  return { dx, dy };
}

function pointInElement(el: Element, clientX: number, clientY: number): boolean {
  const box = el.getBoundingClientRect();
  return clientX >= box.left && clientX <= box.right && clientY >= box.top && clientY <= box.bottom;
}

function scrollFromPoint(clientX: number, clientY: number, deltaX: number, deltaY: number): boolean {
  const stack =
    typeof document.elementsFromPoint === "function"
      ? document.elementsFromPoint(clientX, clientY)
      : ([document.elementFromPoint(clientX, clientY)].filter(Boolean) as Element[]);
  for (const root of stack) {
    let el: Element | null = root;
    while (el && el !== document.documentElement) {
      if (deltaY !== 0 && overflowScrollableY(el)) {
        el.scrollTop += deltaY;
        return true;
      }
      if (deltaX !== 0 && overflowScrollableX(el)) {
        el.scrollLeft += deltaX;
        return true;
      }
      el = el.parentElement;
    }
  }
  const fallback = document.querySelectorAll(
    ".secure-app-window-body,.host-app-panel,.secure-taskbar-notify-flyout,.secure-access-points,.secure-events",
  );
  for (const node of fallback) {
    if (!pointInElement(node, clientX, clientY)) {
      continue;
    }
    if (deltaY !== 0 && overflowScrollableY(node)) {
      node.scrollTop += deltaY;
      return true;
    }
    if (deltaX !== 0 && overflowScrollableX(node)) {
      node.scrollLeft += deltaX;
      return true;
    }
  }
  return false;
}

/** DOM wheel walker for viewport-locked Secure Shell (native kiosk + Vite). */
export function installViewportLockedWheelScroll(): () => void {
  const onWheel = (event: WheelEvent): void => {
    if (event.defaultPrevented) {
      return;
    }
    const { dx, dy } = wheelDeltaPixels(event);
    if (dx === 0 && dy === 0) {
      return;
    }
    const moved = scrollFromPoint(event.clientX, event.clientY, dx, dy);
    if (moved) {
      event.preventDefault();
    }
  };
  window.addEventListener("wheel", onWheel, { passive: false, capture: true });
  return () => window.removeEventListener("wheel", onWheel, { capture: true });
}
