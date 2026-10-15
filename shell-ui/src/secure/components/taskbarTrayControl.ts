import { runtimeRequest } from "../nativeTransport";

export const SEAT_PERCENT_MIN = 10;
export const SEAT_PERCENT_MAX = 100;
export const SEAT_PERCENT_STEP = 5;

export function clampSeatPercent(value: number): number {
  const snapped = Math.round(value / SEAT_PERCENT_STEP) * SEAT_PERCENT_STEP;
  return Math.max(SEAT_PERCENT_MIN, Math.min(SEAT_PERCENT_MAX, snapped));
}

export function wheelSeatPercent(current: number, deltaY: number): number {
  return clampSeatPercent(current + (deltaY < 0 ? SEAT_PERCENT_STEP : -SEAT_PERCENT_STEP));
}

export function audioControlAvailable(
  snapshot: { capabilities: Array<{ id: string; availability: string }> } | null,
): boolean {
  return (
    snapshot?.capabilities.some(
      (item) => item.id === "audio.control" && item.availability === "available",
    ) === true
  );
}

export async function setSeatVolume(percent: number): Promise<void> {
  const response = await runtimeRequest(
    "audio.control",
    {
      action: "set_volume",
      percent: clampSeatPercent(percent),
      name: "Speaker",
    },
    8_000,
  );
  if (!response.ok) {
    throw new Error(response.error?.message ?? response.status);
  }
}
