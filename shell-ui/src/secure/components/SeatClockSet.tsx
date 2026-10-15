import { useMemo, useState } from "react";
import { DetailsSpoiler } from "../design/controls";
import { localizedValue, type LocaleStrings } from "../locale";
import { runtimeRequest } from "../nativeTransport";

function localStamp(): string {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
}

function toWallClock(value: string): string | null {
  const match = /^(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2})(?::\d{2})?$/.exec(value.trim());
  if (!match) {
    return null;
  }
  return `${match[1]} ${match[2]}`;
}

export function SeatClockSet({ strings }: { strings: LocaleStrings }) {
  const initial = useMemo(localStamp, []);
  const [value, setValue] = useState(initial);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [technical, setTechnical] = useState<string | null>(null);

  const apply = async (): Promise<void> => {
    const datetime = toWallClock(value);
    setMessage(null);
    setTechnical(null);
    if (!datetime) {
      setMessage(strings.operationFailed);
      setTechnical("datetime_invalid");
      return;
    }
    setBusy(true);
    try {
      const response = await runtimeRequest(
        "time.sync",
        { action: "set", datetime },
        20_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.clockSetApplied);
    } catch (caught) {
      setMessage(strings.operationFailed);
      setTechnical(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="secure-power-actions">
      <label>
        {strings.clockSetLabel}
        <input
          type="datetime-local"
          value={value}
          disabled={busy}
          onChange={(event) => setValue(event.target.value)}
        />
      </label>
      <div className="secure-inline-actions">
        <button type="button" disabled={busy} onClick={() => void apply()}>
          {busy ? strings.refreshing : strings.setClock}
        </button>
      </div>
      {message && <p className="secure-detail">{message}</p>}
      <DetailsSpoiler summary={strings.moreDetails}>
        <p className="secure-detail">{strings.timeSetHint}</p>
        {technical ? (
          <p className="secure-detail">{localizedValue(strings, technical)}</p>
        ) : null}
      </DetailsSpoiler>
    </div>
  );
}
