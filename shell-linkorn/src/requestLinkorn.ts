import { isLinkornAction, type LinkornAction } from "./seat";

interface LinkornHandler {
  postMessage(action: LinkornAction): void;
}

declare global {
  interface Window {
    webkit?: {
      messageHandlers?: {
        linkornew?: LinkornHandler;
      };
    };
  }
}

function handler(): LinkornHandler | null {
  const seat = window.webkit?.messageHandlers?.linkornew;
  if (!seat || typeof seat.postMessage !== "function") {
    return null;
  }
  return seat;
}

export function linkornSeatPresent(): boolean {
  return handler() !== null;
}

/** One word. The seat process is the only thing that may spawn the host. */
export function requestLinkorn(action: LinkornAction): void {
  if (!isLinkornAction(action)) {
    throw new Error("linkornew_seat_absent");
  }
  const seat = handler();
  if (!seat) {
    throw new Error("linkornew_seat_absent");
  }
  seat.postMessage(action);
}
