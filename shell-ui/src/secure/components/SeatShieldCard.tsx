import { useCallback, useEffect, useRef, useState } from "react";
import { DetailsSpoiler, TextField } from "../design/controls";
import { HostAppFact, HostAppShell } from "../design/HostAppShell";
import { runtimeRequest } from "../nativeTransport";
import type { LocaleStrings } from "../locale";
import {
  formatSeatShieldObserveMessage,
  mapSeatShieldFeedStatus,
  mapSeatShieldPolicy,
  mapSeatShieldEvents,
  mapSeatShieldJournal,
  mapSeatShieldQuarantineList,
  mapSeatShieldStatus,
  otaBaselineLocaleKey,
  type SeatShieldFeedStatusView,
  type SeatShieldPolicyView,
  type SeatShieldEventView,
  type SeatShieldJournalGroupBy,
  type SeatShieldJournalView,
  type SeatShieldQuarantineItemView,
  type SeatShieldStatusView,
} from "../../session/seatShield";
import {
  buildDisableActsParams,
  buildDisableAutoQuarantineParams,
  buildEnableActsParams,
  buildEnableAutoQuarantineParams,
  buildQuarantineFileParams,
  buildResetParams,
  buildRestoreQuarantineParams,
  buildScanPathParams,
  buildSeatShieldActParams,
  buildSetAllowlistParams,
  buildSetAutoActProtectedParams,
  buildSetHeuristicsParams,
  CONFIRM_BLOCK_PEER,
  CONFIRM_ENABLE_ACTS,
  CONFIRM_ENABLE_QUARANTINE,
  CONFIRM_QUARANTINE_FILE,
  CONFIRM_RESET_SEAT_SHIELD,
  CONFIRM_RESTORE_QUARANTINE,
  CONFIRM_UNBLOCK_PEER,
  seatShieldActsVisible,
  seatShieldArmVisible,
  seatShieldDisarmVisible,
  type SeatShieldActAction,
} from "../../session/seatShieldActConfirm";

interface Props {
  strings: LocaleStrings;
}

type ShieldTab = "overview" | "connections" | "journal" | "blocks" | "files" | "settings";

const POLL_MS = 20_000;

function modeLabel(strings: LocaleStrings, mode: string, actEnabled: boolean): string {
  switch (mode) {
    case "observe_only":
      return strings.seatShieldModeObserveOnly;
    case "armed":
      return strings.seatShieldModeArmed;
    case "unavailable":
      return actEnabled
        ? strings.seatShieldModeFloorMissing
        : strings.seatShieldModeUnavailable;
    default:
      return mode;
  }
}

function localizedOtaBaseline(strings: LocaleStrings, status: string): string {
  const key = otaBaselineLocaleKey(status);
  const value = strings[key as keyof LocaleStrings];
  return typeof value === "string" ? value : status;
}

function eventKindLabel(strings: LocaleStrings, kind: string): string {
  switch (kind) {
    case "network_suspicious_port":
      return strings.seatShieldKindSuspiciousPort;
    case "possible_port_scan":
      return strings.seatShieldKindPortScan;
    case "kernel_taint":
      return strings.seatShieldKindKernelTaint;
    case "act_applied":
      return strings.seatShieldKindActApplied;
    case "ioc_hash_match":
      return strings.seatShieldKindIocHash;
    case "peer_probe_protected_service":
      return strings.seatShieldKindPeerProbe;
    case "suspicious_exec_location":
      return strings.seatShieldKindSuspiciousExec;
    case "ioc_peer_match":
      return strings.seatShieldKindIocPeer;
    case "act_preview":
      return strings.seatShieldKindActPreview;
    case "act_denied":
      return strings.seatShieldKindActDenied;
    case "policy_suppressed":
      return strings.seatShieldKindPolicySuppressed;
    case "backend_unavailable":
      return strings.seatShieldKindBackendUnavailable;
    case "file_scan_hit":
      return strings.seatShieldKindFileScanHit;
    case "file_scan_denied":
      return strings.seatShieldKindFileScanDenied;
    case "file_quarantined":
      return strings.seatShieldKindFileQuarantined;
    case "file_restored":
      return strings.seatShieldKindFileRestored;
    case "file_watch_seen":
      return strings.seatShieldKindFileWatchSeen;
    case "chain_scan_then_probe":
      return strings.seatShieldKindChainScanProbe;
    case "chain_ioc_peer_probe":
      return strings.seatShieldKindChainIocProbe;
    case "chain_watch_ioc_live":
      return strings.seatShieldKindChainWatchLive;
    case "chain_tmp_exec_net":
      return strings.seatShieldKindChainTmpExec;
    case "chain_multi_signal":
      return strings.seatShieldKindChainMulti;
    case "heuristic_reconnect_burst":
      return strings.seatShieldKindReconnectBurst;
    default:
      return kind;
  }
}

function backendLabel(strings: LocaleStrings, backend: string): string {
  switch (backend) {
    case "unavailable":
      return strings.seatShieldBackendUnavailable;
    case "recording":
      return strings.seatShieldBackendRecording;
    case "helper":
      return strings.seatShieldBackendHelper;
    case "proc_net_tcp":
      return strings.seatShieldBackendProc;
    case "probe_inet_diag":
      return strings.seatShieldBackendProbe;
    default:
      return backend;
  }
}

export function SeatShieldCard({ strings }: Props) {
  const [tab, setTab] = useState<ShieldTab>("overview");
  const [status, setStatus] = useState<SeatShieldStatusView | null>(null);
  const [events, setEvents] = useState<SeatShieldEventView[]>([]);
  const [journal, setJournal] = useState<SeatShieldJournalView | null>(null);
  const [journalQuery, setJournalQuery] = useState("");
  const [journalGroupBy, setJournalGroupBy] = useState<SeatShieldJournalGroupBy>("none");
  const journalQueryRef = useRef(journalQuery);
  const journalGroupByRef = useRef(journalGroupBy);
  journalQueryRef.current = journalQuery;
  journalGroupByRef.current = journalGroupBy;
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [peerIp, setPeerIp] = useState("");
  const [scanPath, setScanPath] = useState("");
  const [held, setHeld] = useState<SeatShieldQuarantineItemView[]>([]);
  const [confirm, setConfirm] = useState("");
  const [armWithAutoAct, setArmWithAutoAct] = useState(false);
  const [lastAppliedCount, setLastAppliedCount] = useState<number | null>(null);
  const [feedStatus, setFeedStatus] = useState<SeatShieldFeedStatusView | null>(null);
  const [policyView, setPolicyView] = useState<SeatShieldPolicyView | null>(null);
  const [allowlistDraft, setAllowlistDraft] = useState("");
  const [feedPath, setFeedPath] = useState("");

  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      const statusResp = await runtimeRequest<Record<string, unknown>>(
        "seat.shield.status",
        {},
      );
      if (!statusResp.ok || !statusResp.data || typeof statusResp.data !== "object") {
        setStatus(null);
        setEvents([]);
        setJournal(null);
        setHeld([]);
        setError(statusResp.error?.message ?? statusResp.status);
        return;
      }
      setError(null);
      setStatus(mapSeatShieldStatus(statusResp.data as Record<string, unknown>));

      const eventsResp = await runtimeRequest<Record<string, unknown>>(
        "seat.shield.events",
        { after_sequence: 0, limit: 50 },
      );
      if (eventsResp.ok && eventsResp.data && typeof eventsResp.data === "object") {
        setEvents(mapSeatShieldEvents(eventsResp.data as Record<string, unknown>));
      } else {
        setEvents([]);
      }

      const journalParams: Record<string, unknown> = {
        after_sequence: 0,
        limit: 40,
        analytics: true,
      };
      const trimmed = journalQueryRef.current.trim();
      if (trimmed) {
        journalParams.q = trimmed;
      }
      const groupBy = journalGroupByRef.current;
      if (groupBy !== "none") {
        journalParams.group_by = groupBy;
      }
      const journalResp = await runtimeRequest<Record<string, unknown>>(
        "seat.shield.journal",
        journalParams,
      );
      if (journalResp.ok && journalResp.data && typeof journalResp.data === "object") {
        setJournal(mapSeatShieldJournal(journalResp.data as Record<string, unknown>));
      } else {
        setJournal(null);
      }

      const heldResp = await runtimeRequest<Record<string, unknown>>(
        "seat.shield.quarantine.list",
        {},
      );
      if (heldResp.ok && heldResp.data && typeof heldResp.data === "object") {
        setHeld(mapSeatShieldQuarantineList(heldResp.data as Record<string, unknown>));
      } else {
        setHeld([]);
      }

      const feedResp = await runtimeRequest<Record<string, unknown>>(
        "seat.shield.feed_status",
        {},
      );
      if (feedResp.ok && feedResp.data && typeof feedResp.data === "object") {
        setFeedStatus(mapSeatShieldFeedStatus(feedResp.data as Record<string, unknown>));
      } else {
        setFeedStatus(null);
      }

      const policyResp = await runtimeRequest<Record<string, unknown>>(
        "seat.shield.policy",
        {},
      );
      if (policyResp.ok && policyResp.data && typeof policyResp.data === "object") {
        setPolicyView(mapSeatShieldPolicy(policyResp.data as Record<string, unknown>));
      } else {
        setPolicyView(null);
      }
    } catch (err) {
      setStatus(null);
      setEvents([]);
      setJournal(null);
      setHeld([]);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  const observe = useCallback(async () => {
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const resp = await runtimeRequest<Record<string, unknown>>("seat.shield.observe", {});
      const data = resp.data && typeof resp.data === "object" ? resp.data : null;
      const applied = data?.applied_acts;
      setLastAppliedCount(Array.isArray(applied) ? applied.length : resp.ok ? 0 : null);
      await refresh();
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
        return;
      }
      if (data) {
        setMessage(formatSeatShieldObserveMessage(data, strings));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }, [refresh, strings]);

  const runArm = useCallback(async () => {
    if (confirm !== CONFIRM_ENABLE_ACTS) {
      setError(strings.seatShieldConfirmMismatch);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const params = buildEnableActsParams(confirm, armWithAutoAct);
      const resp = await runtimeRequest("seat.shield.policy.set", params);
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
      } else {
        setConfirm("");
      }
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }, [armWithAutoAct, confirm, refresh, strings.seatShieldConfirmMismatch]);

  const runSetAutoAct = useCallback(
    async (enabled: boolean) => {
      if (!status?.actEnabled) {
        setError(strings.seatShieldAutoActNeedsArm);
        return;
      }
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const resp = await runtimeRequest(
          "seat.shield.policy.set",
          buildSetAutoActProtectedParams(enabled),
        );
        if (!resp.ok) {
          setError(resp.error?.message ?? resp.status);
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setBusy(false);
      }
    },
    [refresh, status?.actEnabled, strings.seatShieldAutoActNeedsArm],
  );

  const runDisarm = useCallback(async () => {
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const resp = await runtimeRequest("seat.shield.policy.set", buildDisableActsParams());
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
      }
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }, [refresh]);

  const runReset = useCallback(async () => {
    if (confirm !== CONFIRM_RESET_SEAT_SHIELD) {
      setError(strings.seatShieldConfirmMismatch);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const params = buildResetParams(confirm);
      const resp = await runtimeRequest("seat.shield.reset", params);
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
      } else {
        setConfirm("");
        setPeerIp("");
      }
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }, [confirm, refresh, strings.seatShieldConfirmMismatch]);

  const runPreview = useCallback(async () => {
    const peer = peerIp.trim();
    if (!peer) {
      setError(strings.seatShieldPeerRequired);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const params = buildSeatShieldActParams({ action: "preview_block", peerIp: peer });
      const resp = await runtimeRequest("seat.shield.act", params);
      if (!resp.ok) {
        setError(resp.error?.message ?? strings.seatShieldPreviewDenied);
      } else {
        setMessage(strings.seatShieldPreviewOk);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, [peerIp, strings.seatShieldPeerRequired, strings.seatShieldPreviewDenied, strings.seatShieldPreviewOk]);

  const runAct = useCallback(
    async (action: Extract<SeatShieldActAction, "block_peer" | "unblock_peer">) => {
      const peer = peerIp.trim();
      if (!peer) {
        setError(strings.seatShieldPeerRequired);
        return;
      }
      const expected = action === "block_peer" ? CONFIRM_BLOCK_PEER : CONFIRM_UNBLOCK_PEER;
      if (confirm !== expected) {
        setError(strings.seatShieldConfirmMismatch);
        return;
      }
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const params = buildSeatShieldActParams({
          action,
          peerIp: peer,
          confirmation: confirm,
        });
        const resp = await runtimeRequest("seat.shield.act", params);
        if (!resp.ok) {
          setError(resp.error?.message ?? resp.status);
        } else {
          setConfirm("");
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setBusy(false);
      }
    },
    [confirm, peerIp, refresh, strings.seatShieldConfirmMismatch, strings.seatShieldPeerRequired],
  );

  const runScanPath = useCallback(async () => {
    const path = scanPath.trim();
    if (!path) {
      setError(strings.seatShieldScanPathRequired);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const resp = await runtimeRequest<Record<string, unknown>>(
        "seat.shield.scan_path",
        buildScanPathParams(path),
      );
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
      } else {
        const hit = resp.data && typeof resp.data === "object" ? Boolean(resp.data.feed_hit) : false;
        setMessage(hit ? strings.seatShieldScanHit : strings.seatShieldScanClean);
      }
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }, [refresh, scanPath, strings.seatShieldScanClean, strings.seatShieldScanHit, strings.seatShieldScanPathRequired]);

  const runHoldFile = useCallback(async () => {
    const path = scanPath.trim();
    if (!path) {
      setError(strings.seatShieldScanPathRequired);
      return;
    }
    if (confirm !== CONFIRM_QUARANTINE_FILE) {
      setError(strings.seatShieldConfirmMismatch);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const resp = await runtimeRequest(
        "seat.shield.quarantine",
        buildQuarantineFileParams(path, confirm),
      );
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
      } else {
        setConfirm("");
        setMessage(strings.seatShieldHoldOk);
      }
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }, [
    confirm,
    refresh,
    scanPath,
    strings.seatShieldConfirmMismatch,
    strings.seatShieldHoldOk,
    strings.seatShieldScanPathRequired,
  ]);

  const runRestoreHeld = useCallback(
    async (sha256: string) => {
      if (confirm !== CONFIRM_RESTORE_QUARANTINE) {
        setError(strings.seatShieldConfirmMismatch);
        return;
      }
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const resp = await runtimeRequest(
          "seat.shield.quarantine.restore",
          buildRestoreQuarantineParams(sha256, confirm),
        );
        if (!resp.ok) {
          setError(resp.error?.message ?? resp.status);
        } else {
          setConfirm("");
          setMessage(strings.seatShieldRestoreOk);
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setBusy(false);
      }
    },
    [confirm, refresh, strings.seatShieldConfirmMismatch, strings.seatShieldRestoreOk],
  );

  const runSetAutoQuarantine = useCallback(
    async (enabled: boolean) => {
      if (enabled && !status?.actEnabled) {
        setError(strings.seatShieldAutoQuarantineNeedsArm);
        return;
      }
      if (enabled && confirm !== CONFIRM_ENABLE_QUARANTINE) {
        setError(strings.seatShieldConfirmMismatch);
        return;
      }
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const params = enabled
          ? buildEnableAutoQuarantineParams(confirm)
          : buildDisableAutoQuarantineParams();
        const resp = await runtimeRequest("seat.shield.policy.set", params);
        if (!resp.ok) {
          setError(resp.error?.message ?? resp.status);
        } else if (enabled) {
          setConfirm("");
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setBusy(false);
      }
    },
    [
      confirm,
      refresh,
      status?.actEnabled,
      strings.seatShieldAutoQuarantineNeedsArm,
      strings.seatShieldConfirmMismatch,
    ],
  );

  const runSetHeuristics = useCallback(
    async (enabled: boolean) => {
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const resp = await runtimeRequest(
          "seat.shield.policy.set",
          buildSetHeuristicsParams(enabled),
        );
        if (!resp.ok) {
          setError(resp.error?.message ?? resp.status);
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setBusy(false);
      }
    },
    [refresh],
  );

  const runUpdateAllowlist = useCallback(
    async (next: string[]) => {
      setBusy(true);
      setError(null);
      setMessage(null);
      try {
        const resp = await runtimeRequest(
          "seat.shield.policy.set",
          buildSetAllowlistParams(next),
        );
        if (!resp.ok) {
          setError(resp.error?.message ?? resp.status);
        } else {
          setMessage(strings.seatShieldAllowlistSaved);
        }
        await refresh();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setBusy(false);
      }
    },
    [refresh, strings.seatShieldAllowlistSaved],
  );

  const runAddAllowlistIp = useCallback(async () => {
    const current = policyView?.allowlist ?? [];
    const ip = allowlistDraft.trim();
    if (!ip) {
      setError(strings.seatShieldAllowlistIpRequired);
      return;
    }
    if (current.includes(ip)) {
      setError(strings.seatShieldAllowlistDuplicate);
      return;
    }
    setAllowlistDraft("");
    await runUpdateAllowlist([...current, ip]);
  }, [
    allowlistDraft,
    policyView?.allowlist,
    runUpdateAllowlist,
    strings.seatShieldAllowlistDuplicate,
    strings.seatShieldAllowlistIpRequired,
  ]);

  const runRemoveAllowlistIp = useCallback(
    async (ip: string) => {
      const current = policyView?.allowlist ?? [];
      await runUpdateAllowlist(current.filter((entry) => entry !== ip));
    },
    [policyView?.allowlist, runUpdateAllowlist],
  );

  const runFeedInstall = useCallback(async () => {
    const path = feedPath.trim();
    if (!path.startsWith("/")) {
      setError(strings.seatShieldFeedPathRequired);
      return;
    }
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const resp = await runtimeRequest("seat.shield.feed.install", { path });
      if (!resp.ok) {
        setError(resp.error?.message ?? resp.status);
      } else {
        setFeedPath("");
        setMessage(strings.seatShieldFeedInstalled);
      }
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }, [feedPath, refresh, strings.seatShieldFeedInstalled, strings.seatShieldFeedPathRequired]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const id = window.setInterval(() => {
      void refresh();
    }, POLL_MS);
    return () => window.clearInterval(id);
  }, [refresh]);

  if (!status) {
    return (
      <HostAppShell
        ariaLabel={strings.seatShield}
        className="seat-shield-root"
        error={error ?? strings.seatShieldUnreachable}
      >
        <button type="button" disabled={busy} onClick={() => void refresh()}>
          {strings.seatShieldRefresh}
        </button>
      </HostAppShell>
    );
  }

  const mode = modeLabel(strings, status.mode, status.actEnabled);
  const backend = backendLabel(strings, status.backend);
  const allowlist = policyView?.allowlist ?? [];
  const showOtaAlert =
    status.otaBaseline === "mismatch" || status.otaBaseline === "stale";
  const showFloorAlert = status.mode === "unavailable" && status.actEnabled;
  const armVisible = seatShieldArmVisible(status.actEnabled);
  const disarmVisible = seatShieldDisarmVisible(status.actEnabled);
  const actsVisible = seatShieldActsVisible(status.mode, status.backend);
  const highEvents = events.filter(
    (event) => event.severity === "high" || event.severity === "critical",
  );
  const overviewThreats = (highEvents.length > 0 ? highEvents : events).slice(0, 3);
  const journalItems = journal?.items ?? [];
  const tabs: { id: ShieldTab; label: string; count?: number }[] = [
    { id: "overview", label: strings.seatShieldTabOverview },
    {
      id: "connections",
      label: strings.seatShieldTabConnections,
      count: status.connectionCount > 0 ? status.connectionCount : undefined,
    },
    {
      id: "journal",
      label: strings.seatShieldTabJournal,
      count: status.openHighEvents > 0 ? status.openHighEvents : undefined,
    },
    {
      id: "blocks",
      label: strings.seatShieldTabBlocks,
      count: status.blockedCount > 0 ? status.blockedCount : undefined,
    },
    {
      id: "files",
      label: strings.seatShieldTabFiles,
      count: status.quarantineCount > 0 ? status.quarantineCount : undefined,
    },
    { id: "settings", label: strings.seatShieldTabSettings },
  ];

  return (
    <HostAppShell
      ariaLabel={strings.seatShield}
      className="seat-shield-root"
      tabs={tabs}
      activeTab={tab}
      onTabChange={(id) => {
        setError(null);
        setMessage(null);
        setTab(id as ShieldTab);
      }}
      busy={busy}
      message={message}
      error={error}
      statusChips={[
        { label: mode, state: status.mode === "armed" ? "connected" : "local" },
        { label: backend, state: status.backend === "helper" ? "connected" : "unavailable" },
        {
          label: status.heuristicsEnabled
            ? strings.seatShieldHeurChipOn
            : strings.seatShieldHeurChipOff,
          state: status.heuristicsEnabled ? "connected" : "unavailable",
        },
        {
          label: `${strings.seatShieldOtaBaseline}: ${localizedOtaBaseline(strings, status.otaBaseline)}`,
          state:
            status.otaBaseline === "ok"
              ? "connected"
              : status.otaBaseline === "missing"
                ? "local"
                : "unavailable",
        },
        {
          label: strings.seatShieldTwin,
          state: status.integrityTwinState === "disabled" ? "unavailable" : "connected",
        },
      ]}
      facts={
        <>
          <HostAppFact label={strings.seatShieldHighEvents} value={status.openHighEvents} />
          <HostAppFact label={strings.seatShieldBlocked} value={status.blockedCount} />
          <HostAppFact label={strings.seatShieldHeld} value={status.quarantineCount} />
          <HostAppFact label={strings.seatShieldWatchPending} value={status.watchPending} />
          <HostAppFact label={strings.seatShieldConnections} value={status.connectionCount} />
        </>
      }
    >
      {tab === "overview" ? (
        <div className="seat-shield-panel">
          {showFloorAlert ? (
            <div className="seat-shield-alert" role="status">
              <p>{strings.seatShieldFloorMissingLead}</p>
            </div>
          ) : null}
          {showOtaAlert ? (
            <div className="seat-shield-alert seat-shield-alert-warn" role="alert">
              <p>{strings.seatShieldOtaBaselineMismatchLead}</p>
            </div>
          ) : null}
          {status.integrityTwinState === "disabled" ? (
            <p className="secure-detail">{strings.seatShieldTwinDisabledLead}</p>
          ) : null}
          {status.heuristicsEnabled ? (
            <>
              <p className="secure-detail">
                {strings.seatShieldChains} ({status.openChains})
              </p>
              {status.openChainTeasers.length === 0 ? (
                <p className="seat-shield-empty">{strings.seatShieldNoChains}</p>
              ) : (
                <ul className="secure-events">
                  {status.openChainTeasers.slice(0, 5).map((chain) => (
                    <li key={chain.chainId}>
                      <strong>{eventKindLabel(strings, chain.kind)}</strong>
                      <span>
                        {chain.ip} · {strings.seatShieldRisk} {chain.riskMilli}
                      </span>
                      <button
                        type="button"
                        disabled={busy}
                        onClick={() => {
                          setPeerIp(chain.ip);
                          setJournalQuery(chain.ip);
                          setTab("journal");
                        }}
                      >
                        {strings.seatShieldOpenEvents}
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </>
          ) : null}
          {overviewThreats.length === 0 ? (
            <p className="seat-shield-empty">{strings.seatShieldNoThreats}</p>
          ) : (
            <ul className="secure-events">
              {overviewThreats.map((event) => (
                <li key={`${event.sequence}-${event.kind}`}>
                  <strong>{eventKindLabel(strings, event.kind)}</strong>
                  <span>{event.summary}</span>
                  {event.entityIp ? (
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => {
                        setPeerIp(event.entityIp ?? "");
                        setTab("blocks");
                      }}
                    >
                      {strings.seatShieldUsePeer}: {event.entityIp}
                    </button>
                  ) : null}
                </li>
              ))}
            </ul>
          )}
          <div className="secure-actions">
            <button type="button" disabled={busy} onClick={() => void refresh()}>
              {strings.seatShieldRefresh}
            </button>
            <button type="button" disabled={busy} onClick={() => void observe()}>
              {strings.seatShieldObserve}
            </button>
            <button type="button" disabled={busy} onClick={() => setTab("connections")}>
              {strings.seatShieldOpenConnections}
            </button>
            {highEvents.length > 0 ? (
              <button type="button" disabled={busy} onClick={() => setTab("journal")}>
                {strings.seatShieldOpenEvents}
              </button>
            ) : null}
          </div>
          {lastAppliedCount != null ? (
            <p className="secure-detail">
              {strings.seatShieldAppliedActs}: {lastAppliedCount}
            </p>
          ) : null}
          {status.backend === "unavailable" ? (
            <p className="secure-detail">{strings.seatShieldHelperMissing}</p>
          ) : null}
          {status.actEnabled && status.backend === "helper" ? (
            <p className="secure-detail">{strings.seatShieldActsReady}</p>
          ) : null}
        </div>
      ) : null}

      {tab === "connections" ? (
        <div className="seat-shield-panel">
          <div className="seat-shield-facts">
            <div className="secure-fact">
              <span>{strings.seatShieldConnections}</span>
              <strong>{status.connectionCount}</strong>
            </div>
            <div className="secure-fact">
              <span>{strings.seatShieldLastObserve}</span>
              <strong>
                {status.connectionsEpoch > 0
                  ? new Date(status.connectionsEpoch * 1000).toLocaleString()
                  : "—"}
              </strong>
            </div>
          </div>
          {status.connectionsTruncated ? (
            <p className="secure-detail">{strings.seatShieldConnectionsTruncated}</p>
          ) : null}
          {status.connections.length === 0 ? (
            <p className="seat-shield-empty">{strings.seatShieldNoConnections}</p>
          ) : (
            <div className="seat-shield-table-wrap">
              <table className="seat-shield-table">
                <thead>
                  <tr>
                    <th>IP</th>
                    <th>L</th>
                    <th>R</th>
                    <th>proc</th>
                    <th>pid</th>
                    <th>{strings.seatShieldRtt}</th>
                    {status.heuristicsEnabled ? <th>{strings.seatShieldRisk}</th> : null}
                  </tr>
                </thead>
                <tbody>
                  {status.connections.map((conn, index) => (
                    <tr key={`${conn.remoteIp}-${conn.localPort}-${conn.pid}-${index}`}>
                      <td>
                        <button
                          type="button"
                          disabled={busy || !conn.remoteIp}
                          onClick={() => {
                            setPeerIp(conn.remoteIp);
                            setTab("blocks");
                          }}
                        >
                          {conn.remoteIp || "—"}
                        </button>
                      </td>
                      <td>{conn.localPort || "—"}</td>
                      <td>{conn.remotePort || "—"}</td>
                      <td>{conn.processName || "—"}</td>
                      <td>{conn.pid || "—"}</td>
                      <td>{conn.rttMs != null ? conn.rttMs : "—"}</td>
                      {status.heuristicsEnabled ? <td>{conn.riskMilli}</td> : null}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <div className="secure-actions">
            <button type="button" disabled={busy} onClick={() => void observe()}>
              {strings.seatShieldObserve}
            </button>
            <button type="button" disabled={busy} onClick={() => void refresh()}>
              {strings.seatShieldRefresh}
            </button>
          </div>
        </div>
      ) : null}

      {tab === "journal" ? (
        <div className="seat-shield-panel">
          <TextField
            label={strings.seatShieldJournalFilter}
            value={journalQuery}
            disabled={busy}
            onChange={setJournalQuery}
          />
          <div className="secure-inline-actions">
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                journalQueryRef.current = journalQuery;
                void refresh();
              }}
            >
              {strings.seatShieldJournalApplyFilter}
            </button>
          </div>
          <div className="seat-shield-journal-group" role="group" aria-label={strings.seatShieldJournalGroupBy}>
            {(
              [
                ["none", strings.seatShieldJournalGroupNone],
                ["chain", strings.seatShieldJournalGroupChain],
                ["kind", strings.seatShieldJournalGroupKind],
                ["severity", strings.seatShieldJournalGroupSeverity],
                ["entity_ip", strings.seatShieldJournalGroupEntityIp],
              ] as const
            ).map(([id, label]) => (
              <button
                key={id}
                type="button"
                className={journalGroupBy === id ? "is-active" : undefined}
                disabled={busy}
                onClick={() => {
                  journalGroupByRef.current = id;
                  setJournalGroupBy(id);
                  void refresh();
                }}
              >
                {label}
              </button>
            ))}
          </div>
          <div className="seat-shield-facts">
            <div className="secure-fact">
              <span>{strings.seatShieldJournalLines}</span>
              <strong>{journal?.journalLines ?? 0}</strong>
            </div>
            {journal?.analytics ? (
              <>
                <div className="secure-fact">
                  <span>{strings.seatShieldJournalAnalytics}</span>
                  <strong>
                    C{journal.analytics.critical}/H{journal.analytics.high}/M
                    {journal.analytics.medium}/L{journal.analytics.low}
                  </strong>
                </div>
              </>
            ) : null}
          </div>
          {journal?.analytics &&
          (journal.analytics.topKinds.length > 0 || journal.analytics.topIps.length > 0) ? (
            <div className="seat-shield-journal-analytics">
              {journal.analytics.topKinds.length > 0 ? (
                <div>
                  <p className="secure-detail">{strings.seatShieldJournalTopKinds}</p>
                  <ul>
                    {journal.analytics.topKinds.slice(0, 8).map((row) => (
                      <li key={row.kind}>
                        <strong>{eventKindLabel(strings, row.kind)}</strong>
                        <span>×{row.count}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
              {journal.analytics.topIps.length > 0 ? (
                <div>
                  <p className="secure-detail">{strings.seatShieldJournalTopIps}</p>
                  <ul>
                    {journal.analytics.topIps.slice(0, 8).map((row) => (
                      <li key={row.ip}>
                        <strong>{row.ip}</strong>
                        <span>×{row.count}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
            </div>
          ) : null}
          {journalGroupBy !== "none" && (journal?.groups.length ?? 0) > 0 ? (
            <div className="seat-shield-journal-groups">
              <p className="secure-detail">{strings.seatShieldJournalGroups}</p>
              <ul>
                {journal!.groups.slice(0, 16).map((group) => (
                  <li key={`${group.key}-${group.latestSequence}`}>
                    <strong>{group.key}</strong>
                    <span>×{group.count}</span>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          {journalItems.length === 0 ? (
            <p className="seat-shield-empty">{strings.seatShieldNoEvents}</p>
          ) : (
            <ol className="secure-events">
              {journalItems.slice(0, 40).map((event) => (
                <li key={`${event.sequence}-${event.kind}-${event.entityIp ?? ""}`}>
                  <strong>{eventKindLabel(strings, event.kind)}</strong>
                  <span>
                    {event.severity} · {event.summary}
                  </span>
                  {event.chainId ? (
                    <p className="secure-detail">
                      {strings.seatShieldJournalChainId}: {event.chainId}
                    </p>
                  ) : null}
                  {event.entityIp ? (
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => {
                        setPeerIp(event.entityIp ?? "");
                        setTab("blocks");
                      }}
                    >
                      {strings.seatShieldUsePeer}: {event.entityIp}
                    </button>
                  ) : null}
                  {event.explainWhy ? (
                    <p className="secure-detail">
                      {strings.seatShieldWhy}: {event.explainWhy}
                    </p>
                  ) : null}
                  {event.explainAction ? (
                    <p className="secure-detail">
                      {strings.seatShieldSuggestedAction}: {event.explainAction}
                    </p>
                  ) : null}
                </li>
              ))}
            </ol>
          )}
          <div className="secure-actions">
            <button type="button" disabled={busy} onClick={() => void refresh()}>
              {strings.seatShieldRefresh}
            </button>
          </div>
        </div>
      ) : null}

      {tab === "blocks" ? (
        <div className="seat-shield-panel">
          <TextField
            label={strings.seatShieldPeerIp}
            value={peerIp}
            disabled={busy}
            onChange={setPeerIp}
          />
          <TextField
            label={strings.seatShieldConfirmLabel}
            value={confirm}
            disabled={busy}
            onChange={setConfirm}
          />
          <DetailsSpoiler summary={strings.moreDetails}>
            <p className="secure-detail">{strings.seatShieldBlocksConfirmHint}</p>
          </DetailsSpoiler>
          <div className="secure-inline-actions">
            <button type="button" disabled={busy} onClick={() => void runPreview()}>
              {strings.seatShieldPreview}
            </button>
            {actsVisible ? (
              <>
                <button type="button" disabled={busy} onClick={() => void runAct("block_peer")}>
                  {strings.seatShieldBlock}
                </button>
                <button type="button" disabled={busy} onClick={() => void runAct("unblock_peer")}>
                  {strings.seatShieldUnblock}
                </button>
              </>
            ) : (
              <p className="secure-detail">{strings.seatShieldBlocksNeedArm}</p>
            )}
          </div>
          <p className="secure-detail">
            {strings.seatShieldBlockedPeers} ({status.blockedCount})
          </p>
          {status.blockedPeers.length === 0 ? (
            <p className="seat-shield-empty">{strings.seatShieldNoBlocked}</p>
          ) : (
            <div className="secure-inline-actions">
              {status.blockedPeers.map((peer) => (
                <button key={peer} type="button" disabled={busy} onClick={() => setPeerIp(peer)}>
                  {peer}
                </button>
              ))}
            </div>
          )}
        </div>
      ) : null}

      {tab === "files" ? (
        <div className="seat-shield-panel">
          <p className="secure-detail">{strings.seatShieldScanRoots}</p>
          <TextField
            label={strings.seatShieldScanPath}
            value={scanPath}
            disabled={busy}
            onChange={setScanPath}
          />
          <TextField
            label={strings.seatShieldConfirmLabel}
            value={confirm}
            disabled={busy}
            onChange={setConfirm}
          />
          <DetailsSpoiler summary={strings.moreDetails}>
            <p className="secure-detail">{strings.seatShieldFilesConfirmHint}</p>
          </DetailsSpoiler>
          <div className="secure-inline-actions">
            <button type="button" disabled={busy} onClick={() => void runScanPath()}>
              {strings.seatShieldScan}
            </button>
            <button type="button" disabled={busy} onClick={() => void runHoldFile()}>
              {strings.seatShieldHold}
            </button>
          </div>
          <p className="secure-detail">
            {strings.seatShieldHeld} ({status.quarantineCount})
          </p>
          {held.length === 0 ? (
            <p className="seat-shield-empty">{strings.seatShieldNoHeld}</p>
          ) : (
            <ol>
              {held.slice(0, 16).map((item) => (
                <li key={item.sha256}>
                  {item.origBasename} · {item.reason} · {item.size}
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => void runRestoreHeld(item.sha256)}
                  >
                    {strings.seatShieldRestore}
                  </button>
                </li>
              ))}
            </ol>
          )}
        </div>
      ) : null}

      {tab === "settings" ? (
        <div className="seat-shield-panel">
          <p className="secure-detail">{strings.seatShieldSettingsLead}</p>
          <div className="seat-shield-facts">
            <div className="secure-fact">
              <span>{strings.seatShieldAutoAct}</span>
              <strong>
                {status.autoActProtected ? strings.seatShieldAutoActOn : strings.seatShieldAutoActOff}
              </strong>
            </div>
            <div className="secure-fact">
              <span>{strings.seatShieldAutoQuarantine}</span>
              <strong>
                {status.autoQuarantineHash
                  ? strings.seatShieldAutoQuarantineOn
                  : strings.seatShieldAutoQuarantineOff}
              </strong>
            </div>
            <div className="secure-fact">
              <span>{strings.seatShieldHeuristics}</span>
              <strong>
                {status.heuristicsEnabled
                  ? strings.seatShieldHeuristicsOn
                  : strings.seatShieldHeuristicsOff}
              </strong>
            </div>
            <div className="secure-fact">
              <span>{strings.seatShieldCycleInterval}</span>
              <strong>{status.cycleIntervalSecs}s</strong>
            </div>
            <div className="secure-fact">
              <span>{strings.seatShieldPorts}</span>
              <strong>{status.protectedPorts.join(", ") || "—"}</strong>
            </div>
            <div className="secure-fact">
              <span>{strings.seatShieldAllowlist}</span>
              <strong>{allowlist.length}</strong>
            </div>
          </div>
          <section className="seat-shield-subpanel" aria-label={strings.seatShieldFeedSection}>
            <h3>{strings.seatShieldFeedSection}</h3>
            {feedStatus ? (
              <>
                <p className="secure-detail">
                  {feedStatus.present
                    ? strings.seatShieldFeedPresent
                        .replace("{version}", feedStatus.version ?? "—")
                        .replace("{ips}", String(feedStatus.ipCount))
                        .replace("{hashes}", String(feedStatus.hashCount))
                    : strings.seatShieldFeedAbsent}
                </p>
                {feedStatus.digest ? (
                  <p className="secure-detail seat-shield-mono">{feedStatus.digest}</p>
                ) : null}
              </>
            ) : (
              <p className="secure-detail">{strings.seatShieldFeedUnknown}</p>
            )}
            <TextField
              label={strings.seatShieldFeedPath}
              value={feedPath}
              disabled={busy}
              onChange={setFeedPath}
            />
            <p className="secure-detail">{strings.seatShieldFeedPathHint}</p>
            <button type="button" disabled={busy} onClick={() => void runFeedInstall()}>
              {strings.seatShieldFeedInstall}
            </button>
          </section>
          <section className="seat-shield-subpanel" aria-label={strings.seatShieldAllowlist}>
            <h3>{strings.seatShieldAllowlist}</h3>
            <p className="secure-detail">{strings.seatShieldAllowlistLead}</p>
            {allowlist.length === 0 ? (
              <p className="seat-shield-empty">{strings.seatShieldAllowlistEmpty}</p>
            ) : (
              <ul className="seat-shield-allowlist">
                {allowlist.map((ip) => (
                  <li key={ip}>
                    <span>{ip}</span>
                    <button type="button" disabled={busy} onClick={() => void runRemoveAllowlistIp(ip)}>
                      {strings.seatShieldAllowlistRemove}
                    </button>
                  </li>
                ))}
              </ul>
            )}
            <TextField
              label={strings.seatShieldAllowlistAdd}
              value={allowlistDraft}
              disabled={busy}
              onChange={setAllowlistDraft}
            />
            <button type="button" disabled={busy} onClick={() => void runAddAllowlistIp()}>
              {strings.seatShieldAllowlistAddButton}
            </button>
          </section>
          <TextField
            label={strings.seatShieldConfirmLabel}
            value={confirm}
            disabled={busy}
            onChange={setConfirm}
          />
          <DetailsSpoiler summary={strings.moreDetails}>
            <p className="secure-detail">{strings.seatShieldSettingsConfirmHint}</p>
          </DetailsSpoiler>
          <div className="secure-inline-actions">
            {armVisible ? (
              <>
                <label className="secure-detail">
                  <input
                    type="checkbox"
                    checked={armWithAutoAct}
                    disabled={busy}
                    onChange={(event) => setArmWithAutoAct(event.target.checked)}
                  />{" "}
                  {strings.seatShieldArmWithAutoAct}
                </label>
                <button type="button" disabled={busy} onClick={() => void runArm()}>
                  {strings.seatShieldArm}
                </button>
              </>
            ) : null}
            {disarmVisible ? (
              <button type="button" disabled={busy} onClick={() => void runDisarm()}>
                {strings.seatShieldDisarm}
              </button>
            ) : null}
            {status.actEnabled && !status.autoActProtected ? (
              <button type="button" disabled={busy} onClick={() => void runSetAutoAct(true)}>
                {strings.seatShieldEnableAutoAct}
              </button>
            ) : null}
            {status.actEnabled && status.autoActProtected ? (
              <button type="button" disabled={busy} onClick={() => void runSetAutoAct(false)}>
                {strings.seatShieldDisableAutoAct}
              </button>
            ) : null}
            {status.actEnabled && !status.autoQuarantineHash ? (
              <button type="button" disabled={busy} onClick={() => void runSetAutoQuarantine(true)}>
                {strings.seatShieldEnableAutoQuarantine}
              </button>
            ) : null}
            {status.actEnabled && status.autoQuarantineHash ? (
              <button type="button" disabled={busy} onClick={() => void runSetAutoQuarantine(false)}>
                {strings.seatShieldDisableAutoQuarantine}
              </button>
            ) : null}
            {status.heuristicsEnabled ? (
              <button type="button" disabled={busy} onClick={() => void runSetHeuristics(false)}>
                {strings.seatShieldDisableHeuristics}
              </button>
            ) : (
              <button type="button" disabled={busy} onClick={() => void runSetHeuristics(true)}>
                {strings.seatShieldEnableHeuristics}
              </button>
            )}
            <button type="button" disabled={busy} onClick={() => void runReset()}>
              {strings.seatShieldReset}
            </button>
          </div>
        </div>
      ) : null}
    </HostAppShell>
  );
}
