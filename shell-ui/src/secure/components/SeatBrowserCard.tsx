import { useCallback, useEffect, useState } from "react";
import { TextField } from "../design/controls";
import { HostAppFact, HostAppShell } from "../design/HostAppShell";
import { nativeBridgeAvailable, runtimeRequest } from "../nativeTransport";
import type { LocaleStrings } from "../locale";
import {
  mapSeatBrowserStatus,
  normalizeAllowHosts,
  normalizePolicyHost,
  type SeatBrowserStatusView,
} from "../../session/seatBrowser";

interface Props {
  strings: LocaleStrings;
}

const VIP_ALLOW = ["app.eliteew.ru", "auth.eliteew.ru", "meet.eliteew.ru"];
const CONFIRM_ENABLE = "ENABLE_BROWSER_POLICY";
const CONFIRM_MEDIA = "ENABLE_BROWSER_MEDIA";

function localAllowlistPreview(allow: string[]): SeatBrowserStatusView {
  return mapSeatBrowserStatus({
    product: "empiieros-seat-browser",
    surface: "shell_overlay",
    host: "webkit_gtk",
    policy_mode: "allowlist",
    policy_fresh: true,
    is_default: false,
    allow,
    allow_count: allow.length,
    deny: [],
    deny_count: 0,
    tab_count: 0,
    tab_cap: 3,
    active_tab: null,
    tabs: [],
    camera: true,
    microphone: true,
    detail: "local preview — native bridge unavailable",
  });
}

function surfaceParams(
  data: Record<string, unknown> | null | undefined,
  fallbackAction: string,
  url?: string,
): Record<string, unknown> {
  const action = typeof data?.host_action === "string" ? data.host_action : fallbackAction;
  const params: Record<string, unknown> = { action };
  const tabId = typeof data?.tab_id === "number" ? data.tab_id : undefined;
  const resolvedUrl = typeof data?.url === "string" ? data.url : url;
  if (resolvedUrl) {
    params.url = resolvedUrl;
  }
  if (tabId && tabId > 0) {
    params.tab_id = tabId;
  }
  return params;
}

export function SeatBrowserCard({ strings }: Props) {
  const [status, setStatus] = useState<SeatBrowserStatusView | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [url, setUrl] = useState("https://app.eliteew.ru/");
  const [evalResult, setEvalResult] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [policyMode, setPolicyMode] = useState<"allowlist" | "open">("allowlist");
  const [draftHosts, setDraftHosts] = useState<string[]>(VIP_ALLOW);
  const [addHost, setAddHost] = useState("");
  const [confirm, setConfirm] = useState("");
  const [mediaConfirm, setMediaConfirm] = useState("");
  const [draftCamera, setDraftCamera] = useState(true);
  const [draftMicrophone, setDraftMicrophone] = useState(true);
  const [tab, setTab] = useState<"browse" | "policy" | "media">("browse");

  const refresh = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const resp = await runtimeRequest<Record<string, unknown>>("seat.browser.status", {});
      if (!resp.ok || !resp.data || typeof resp.data !== "object") {
        if (import.meta.env.DEV && !nativeBridgeAvailable()) {
          setStatus(localAllowlistPreview(VIP_ALLOW));
          setError(null);
          return;
        }
        setStatus(null);
        setError(resp.error?.message ?? resp.status);
        return;
      }
      const view = mapSeatBrowserStatus(resp.data as Record<string, unknown>);
      setStatus(view);
      if (view.allowHostsObserved) {
        setDraftHosts(view.allowHosts);
        setPolicyMode(view.policyMode === "open" ? "open" : "allowlist");
      }
      setDraftCamera(view.camera);
      setDraftMicrophone(view.microphone);
    } catch (err) {
      if (import.meta.env.DEV && !nativeBridgeAvailable()) {
        setStatus(localAllowlistPreview(VIP_ALLOW));
        setError(null);
        return;
      }
      setStatus(null);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  const evaluate = useCallback(async () => {
    const trimmed = url.trim();
    if (!trimmed) {
      setError(strings.seatBrowserUrlRequired);
      return;
    }
    setBusy(true);
    setError(null);
    setEvalResult(null);
    try {
      const resp = await runtimeRequest<Record<string, unknown>>("seat.browser.evaluate", {
        url: trimmed,
      });
      if (!resp.ok || !resp.data || typeof resp.data !== "object") {
        setError(resp.error?.message ?? resp.status);
        return;
      }
      const data = resp.data as Record<string, unknown>;
      const allowed = Boolean(data.allowed);
      const reason = data.reason != null ? String(data.reason) : "";
      setEvalResult(
        allowed
          ? strings.seatBrowserEvalAllow
          : `${strings.seatBrowserEvalDeny}${reason ? ` (${reason})` : ""}`,
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, [url, strings.seatBrowserUrlRequired, strings.seatBrowserEvalAllow, strings.seatBrowserEvalDeny]);

  const applyPolicy = useCallback(
    async (mode: "allowlist" | "open", allow: string[]) => {
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const resp = await runtimeRequest<Record<string, unknown>>("seat.browser.policy.set", {
          mode,
          allow,
          confirmation: CONFIRM_ENABLE,
        });
        if (!resp.ok) {
          setError(resp.error?.message ?? resp.status);
          return;
        }
        setDraftHosts(allow);
        setMessage(strings.seatBrowserPolicyApplied);
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
    [refresh, strings.seatBrowserPolicyApplied],
  );

  const applyMedia = useCallback(async () => {
    const enabling =
      (draftCamera && status?.camera === false) || (draftMicrophone && status?.microphone === false);
    if (enabling && mediaConfirm.trim() !== CONFIRM_MEDIA) {
      setError(strings.seatBrowserMediaConfirm);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const params: Record<string, unknown> = {
        camera: draftCamera,
        microphone: draftMicrophone,
      };
      if (enabling) {
        params.confirmation = CONFIRM_MEDIA;
      }
      const resp = await runtimeRequest<Record<string, unknown>>("seat.browser.media.set", params);
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
        return;
      }
      setMessage(strings.seatBrowserMediaApplied);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, [
    draftCamera,
    draftMicrophone,
    mediaConfirm,
    refresh,
    status?.camera,
    status?.microphone,
    strings.seatBrowserMediaApplied,
    strings.seatBrowserMediaConfirm,
  ]);

  const applyTypedPolicy = useCallback(async () => {
    if (confirm.trim() !== CONFIRM_ENABLE) {
      setError(strings.seatBrowserPolicyConfirm);
      return;
    }
    await applyPolicy(policyMode, normalizeAllowHosts(draftHosts));
  }, [applyPolicy, confirm, draftHosts, policyMode, strings.seatBrowserPolicyConfirm]);

  const applyVip = useCallback(async () => {
    setPolicyMode("allowlist");
    setDraftHosts(VIP_ALLOW);
    setConfirm(CONFIRM_ENABLE);
    await applyPolicy("allowlist", VIP_ALLOW);
  }, [applyPolicy]);

  const addDraftHost = useCallback(() => {
    const host = normalizePolicyHost(addHost);
    if (!host) {
      return;
    }
    setDraftHosts((prev) => normalizeAllowHosts([...prev, host]));
    setAddHost("");
  }, [addHost]);

  const removeDraftHost = useCallback((host: string) => {
    setDraftHosts((prev) => prev.filter((item) => item.toLowerCase() !== host.toLowerCase()));
  }, []);

  const openSurface = useCallback(async () => {
    const trimmed = url.trim();
    if (!trimmed.startsWith("https://")) {
      setError(strings.seatBrowserUrlRequired);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const nav = await runtimeRequest<Record<string, unknown>>("seat.browser.navigate", {
        url: trimmed,
      });
      if (!nav.ok) {
        setError(nav.error?.message ?? nav.status);
        return;
      }
      const surface = await runtimeRequest<Record<string, unknown>>(
        "shell.seat_browser_surface",
        surfaceParams(nav.data, "show", trimmed),
        8_000,
      );
      if (!surface.ok) {
        setError(surface.error?.message ?? surface.status);
        return;
      }
      setMessage(strings.seatBrowserOpened);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, [url, strings.seatBrowserUrlRequired, strings.seatBrowserOpened, refresh]);

  const hostChrome = useCallback(
    async (
      runtimeOp: "seat.browser.reload" | "seat.browser.history",
      surfaceAction: string,
      params: Record<string, unknown> = {},
    ) => {
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const gated = await runtimeRequest<Record<string, unknown>>(runtimeOp, params);
        if (!gated.ok) {
          setError(gated.error?.message ?? gated.status);
          return;
        }
        const surface = await runtimeRequest<Record<string, unknown>>(
          "shell.seat_browser_surface",
          surfaceParams(gated.data, surfaceAction),
          5_000,
        );
        if (!surface.ok) {
          setError(surface.error?.message ?? surface.status);
          return;
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  const closeSurface = useCallback(async () => {
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const closed = await runtimeRequest<Record<string, unknown>>("seat.browser.close", {});
      if (!closed.ok) {
        setError(closed.error?.message ?? closed.status);
        return;
      }
      const surface = await runtimeRequest<Record<string, unknown>>(
        "shell.seat_browser_surface",
        { action: "hide" },
        5_000,
      );
      if (!surface.ok) {
        setError(surface.error?.message ?? surface.status);
        return;
      }
      setMessage(strings.seatBrowserClosed);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, [strings.seatBrowserClosed, refresh]);

  const openTab = useCallback(async () => {
    const trimmed = url.trim();
    const target = trimmed.length === 0 ? "about:blank" : trimmed;
    if (target !== "about:blank" && !target.startsWith("https://")) {
      setError(strings.seatBrowserUrlRequired);
      return;
    }
    if (status && status.tabCount >= status.tabCap) {
      setError(strings.seatBrowserTabLimit);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const opened = await runtimeRequest<Record<string, unknown>>("seat.browser.tab.open", {
        url: target,
      });
      if (!opened.ok) {
        setError(
          opened.error?.message === "tab_limit"
            ? strings.seatBrowserTabLimit
            : (opened.error?.message ?? opened.status),
        );
        return;
      }
      const surface = await runtimeRequest<Record<string, unknown>>(
        "shell.seat_browser_surface",
        surfaceParams(opened.data, "tab_open", target),
        8_000,
      );
      if (!surface.ok) {
        setError(surface.error?.message ?? surface.status);
        return;
      }
      setMessage(strings.seatBrowserOpened);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, [
    refresh,
    status,
    strings.seatBrowserOpened,
    strings.seatBrowserTabLimit,
    strings.seatBrowserUrlRequired,
    url,
  ]);

  const selectTab = useCallback(
    async (id: number) => {
      setBusy(true);
      setError(null);
      try {
        const selected = await runtimeRequest<Record<string, unknown>>("seat.browser.tab.select", {
          id,
        });
        if (!selected.ok) {
          setError(selected.error?.message ?? selected.status);
          return;
        }
        const surface = await runtimeRequest<Record<string, unknown>>(
          "shell.seat_browser_surface",
          surfaceParams(selected.data, "tab_select"),
          5_000,
        );
        if (!surface.ok) {
          setError(surface.error?.message ?? surface.status);
          return;
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  const closeTab = useCallback(
    async (id: number) => {
      setBusy(true);
      setError(null);
      try {
        const closed = await runtimeRequest<Record<string, unknown>>("seat.browser.tab.close", {
          id,
        });
        if (!closed.ok) {
          setError(closed.error?.message ?? closed.status);
          return;
        }
        const surface = await runtimeRequest<Record<string, unknown>>(
          "shell.seat_browser_surface",
          surfaceParams(closed.data, "tab_close"),
          5_000,
        );
        if (!surface.ok) {
          setError(surface.error?.message ?? surface.status);
          return;
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (!status) {
    return (
      <HostAppShell
        ariaLabel={strings.seatBrowser}
        error={error ?? strings.seatBrowserUnreachable}
      >
        <button type="button" disabled={busy} onClick={() => void refresh()}>
          {strings.seatBrowserRefresh}
        </button>
      </HostAppShell>
    );
  }

  const shownHosts = status.allowHostsObserved ? status.allowHosts : draftHosts;
  const shownCount = status.allowHostsObserved ? status.allowCount : draftHosts.length;

  return (
    <HostAppShell
      ariaLabel={strings.seatBrowser}
      tabs={[
        {
          id: "browse",
          label: strings.hostAppTabBrowserBrowse,
          count: status.tabCount > 0 ? status.tabCount : undefined,
        },
        { id: "policy", label: strings.hostAppTabBrowserPolicy },
        { id: "media", label: strings.hostAppTabBrowserMedia },
      ]}
      activeTab={tab}
      onTabChange={(id) => setTab(id as "browse" | "policy" | "media")}
      busy={busy}
      message={message}
      error={error}
      statusChips={[
        {
          label: status.policyMode,
          state: status.policyFresh ? "connected" : "degraded",
        },
        {
          label: `${strings.seatBrowserTabs} ${status.tabCount}/${status.tabCap}`,
          state: status.tabCount > 0 ? "connected" : "local",
        },
        {
          label: `${strings.seatBrowserCamera}/${strings.seatBrowserMicrophone}`,
          state: status.camera || status.microphone ? "connected" : "unavailable",
        },
      ]}
      facts={
        <>
          <HostAppFact label={strings.seatBrowserAllowCount} value={String(shownCount)} />
          <HostAppFact label={strings.seatBrowserTabs} value={`${status.tabCount}/${status.tabCap}`} />
        </>
      }
    >
      {tab === "browse" ? (
        <div className="host-app-section">
          {status.isDefault || !status.policyFresh ? (
            <p className="secure-detail">
              {status.isDefault ? strings.seatBrowserDefaultDeny : ""}
              {!status.policyFresh ? ` · ${strings.seatBrowserStale}` : ""}
            </p>
          ) : null}
          <div className="secure-host-list-block">
            <span>
              {strings.seatBrowserPolicyAllow} ({shownCount})
            </span>
            {shownHosts.length === 0 ? (
              <p className="secure-detail">{strings.seatBrowserAllowEmpty}</p>
            ) : (
              <ul className="secure-host-list">
                {shownHosts.map((host) => (
                  <li key={host}>
                    <code>{host}</code>
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="secure-actions">
            <button type="button" disabled={busy} onClick={() => void refresh()}>
              {strings.seatBrowserRefresh}
            </button>
            <button type="button" disabled={busy} onClick={() => void applyVip()}>
              {strings.seatBrowserPolicyVip}
            </button>
          </div>
          <TextField label={strings.seatBrowserUrl} value={url} disabled={busy} onChange={setUrl} />
          <div className="secure-inline-actions">
            <button type="button" disabled={busy} onClick={() => void evaluate()}>
              {strings.seatBrowserEvaluate}
            </button>
            <button type="button" disabled={busy} onClick={() => void openSurface()}>
              {strings.seatBrowserOpen}
            </button>
            <button
              type="button"
              disabled={busy || status.tabCount >= status.tabCap}
              onClick={() => void openTab()}
            >
              {strings.seatBrowserNewTab}
            </button>
            <button type="button" disabled={busy} onClick={() => void closeSurface()}>
              {strings.seatBrowserClose}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => void hostChrome("seat.browser.history", "back", { direction: "back" })}
            >
              {strings.seatBrowserBack}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() =>
                void hostChrome("seat.browser.history", "forward", { direction: "forward" })
              }
            >
              {strings.seatBrowserForward}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => void hostChrome("seat.browser.reload", "reload")}
            >
              {strings.seatBrowserReload}
            </button>
          </div>
          <div className="secure-host-list-block">
            <span>
              {strings.seatBrowserTabs} ({status.tabCount}/{status.tabCap})
            </span>
            {status.tabs.length === 0 ? (
              <p className="secure-detail">{strings.seatBrowserTabsEmpty}</p>
            ) : (
              <ul className="secure-host-list secure-browser-tabs">
                {status.tabs.map((browserTab) => (
                  <li key={browserTab.id} data-active={status.activeTab === browserTab.id ? "1" : "0"}>
                    <button type="button" disabled={busy} onClick={() => void selectTab(browserTab.id)}>
                      <code>{browserTab.host || strings.seatBrowserTabBlank}</code>
                    </button>
                    <button type="button" disabled={busy} onClick={() => void closeTab(browserTab.id)}>
                      {strings.seatBrowserCloseTab}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="secure-host-list-block">
            <span>
              {strings.seatBrowserDownloads} ({status.downloadCount}/{status.downloadCap})
            </span>
            {status.downloads.length === 0 ? (
              <p className="secure-detail">{strings.seatBrowserDownloadsEmpty}</p>
            ) : (
              <ul className="secure-host-list">
                {status.downloads.map((item) => (
                  <li key={item.id}>
                    <code>
                      {item.name} · {item.host || "—"} · {item.phase} · {item.bytes}
                    </code>
                  </li>
                ))}
              </ul>
            )}
          </div>
          {evalResult ? <p className="secure-detail">{evalResult}</p> : null}
        </div>
      ) : null}

      {tab === "policy" ? (
        <div className="host-app-section">
          <label className="secure-field">
            <span>{strings.seatBrowserPolicyMode}</span>
            <select
              value={policyMode}
              disabled={busy}
              onChange={(event) => setPolicyMode(event.target.value === "open" ? "open" : "allowlist")}
            >
              <option value="allowlist">{strings.seatBrowserPolicyAllowlistMode}</option>
              <option value="open">{strings.seatBrowserPolicyOpenMode}</option>
            </select>
          </label>
          <div className="secure-host-list-block">
            <span>
              {strings.seatBrowserPolicyAllow} ({draftHosts.length})
            </span>
            {draftHosts.length === 0 ? (
              <p className="secure-detail">{strings.seatBrowserAllowEmpty}</p>
            ) : (
              <ul className="secure-host-list">
                {draftHosts.map((host) => (
                  <li key={host}>
                    <code>{host}</code>
                    <button type="button" disabled={busy} onClick={() => removeDraftHost(host)}>
                      {strings.seatBrowserAllowRemove}
                    </button>
                  </li>
                ))}
              </ul>
            )}
            <label className="secure-host-add">
              <input
                type="text"
                value={addHost}
                disabled={busy}
                autoComplete="off"
                spellCheck={false}
                placeholder={strings.seatBrowserAllowAdd}
                onChange={(event) => setAddHost(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    addDraftHost();
                  }
                }}
              />
              <button type="button" disabled={busy || !addHost.trim()} onClick={addDraftHost}>
                {strings.seatBrowserAllowAdd}
              </button>
            </label>
          </div>
          <TextField
            label={strings.seatBrowserPolicyConfirm}
            value={confirm}
            disabled={busy}
            onChange={setConfirm}
          />
          <div className="secure-actions">
            <button type="button" disabled={busy} onClick={() => void applyTypedPolicy()}>
              {strings.seatBrowserPolicyApply}
            </button>
            <button type="button" disabled={busy} onClick={() => void applyVip()}>
              {strings.seatBrowserPolicyVip}
            </button>
          </div>
        </div>
      ) : null}

      {tab === "media" ? (
        <div className="host-app-section">
          <label className="secure-field">
            <input
              type="checkbox"
              checked={draftCamera}
              disabled={busy}
              onChange={(event) => setDraftCamera(event.target.checked)}
            />
            <span>{strings.seatBrowserCamera}</span>
          </label>
          <label className="secure-field">
            <input
              type="checkbox"
              checked={draftMicrophone}
              disabled={busy}
              onChange={(event) => setDraftMicrophone(event.target.checked)}
            />
            <span>{strings.seatBrowserMicrophone}</span>
          </label>
          <TextField
            label={strings.seatBrowserMediaConfirm}
            value={mediaConfirm}
            disabled={busy}
            onChange={setMediaConfirm}
          />
          <div className="secure-actions">
            <button type="button" disabled={busy} onClick={() => void applyMedia()}>
              {strings.seatBrowserMediaApply}
            </button>
          </div>
        </div>
      ) : null}
    </HostAppShell>
  );
}
