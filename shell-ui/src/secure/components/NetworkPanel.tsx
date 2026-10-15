import { useEffect, useMemo, useState } from "react";
import { runtimeRequest } from "../nativeTransport";
import { localizedValue, type LocaleStrings } from "../locale";
import {
  ActionBar,
  DetailsSpoiler,
  SelectField,
  SettingRow,
  SignalMeter,
  StatusPill,
  TextField,
  Toggle,
} from "../design/controls";
import { HostAppFact, HostAppShell } from "../design/HostAppShell";
import { formatDbm, formatMhz, formatMbps } from "../format";
import type {
  NetworkConfigureInput,
  SystemSnapshot,
  WifiAccessPoint,
  WifiScanResult,
} from "../types";

interface Props {
  snapshot: SystemSnapshot;
  strings: LocaleStrings;
  onApplied: () => Promise<void>;
}

export function NetworkPanel({ snapshot, strings, onApplied }: Props) {
  const configurable = snapshot.network.interfaces.filter(
    (item) => item.kind === "ethernet" || item.kind === "wifi",
  );
  const wifiInterfaces = configurable.filter((item) => item.kind === "wifi");
  const associations = snapshot.wifi.data.associations ?? [];
  const associated = associations[0] ?? null;
  const radios = snapshot.wifi.data.radios ?? [];
  const wlanRadio = radios.find((item) => item.kind === "wlan") ?? radios[0] ?? null;
  const radioAvailable = snapshot.capabilities.some(
    (item) => item.id === "network.wifi.radio" && item.availability === "available",
  );
  const [interfaceName, setInterfaceName] = useState(
    associated?.interface ?? configurable[0]?.name ?? "",
  );
  const [useDhcp, setUseDhcp] = useState(true);
  const [address, setAddress] = useState("");
  const [prefix, setPrefix] = useState("24");
  const [gateway, setGateway] = useState("");
  const [dns, setDns] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [technical, setTechnical] = useState<string | null>(null);
  const [accessPoints, setAccessPoints] = useState<WifiAccessPoint[]>([]);
  const [wifiScanned, setWifiScanned] = useState(false);
  const [ssid, setSsid] = useState(associated?.ssid ?? "");
  const [psk, setPsk] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  useEffect(() => {
    if (!configurable.some((item) => item.name === interfaceName)) {
      setInterfaceName(associated?.interface ?? configurable[0]?.name ?? "");
    }
  }, [associated?.interface, configurable, interfaceName]);

  useEffect(() => {
    if (associated?.ssid) {
      setSsid(associated.ssid);
    }
  }, [associated?.ssid]);

  useEffect(() => {
    const selected = configurable.find((item) => item.name === interfaceName);
    const cidr = selected?.ipv4_addresses[0];
    if (!cidr) {
      return;
    }
    const [ip, pfx] = cidr.split("/");
    if (ip) {
      setAddress(ip);
    }
    if (pfx) {
      setPrefix(pfx);
    }
    if (snapshot.network.default_route?.gateway) {
      setGateway(snapshot.network.default_route.gateway);
    }
    if (snapshot.network.dns_servers.length > 0) {
      setDns(snapshot.network.dns_servers.join(", "));
    }
  }, [configurable, interfaceName, snapshot.network.default_route?.gateway, snapshot.network.dns_servers]);

  const selectedAccessPoint = useMemo(
    () => accessPoints.find((item) => item.ssid === ssid) ?? null,
    [accessPoints, ssid],
  );

  const applyNetwork = async (): Promise<void> => {
    if (!interfaceName) return;
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    const input: NetworkConfigureInput = {
      interface: interfaceName,
      method: useDhcp ? "dhcp" : "static",
      dns: dns
        .split(",")
        .map((item) => item.trim())
        .filter(Boolean),
      ...(useDhcp
        ? {}
        : {
            address: address.trim(),
            prefix: Number.parseInt(prefix, 10),
            gateway: gateway.trim() || undefined,
          }),
    };
    try {
      const response = await runtimeRequest("network.configure", input, 30_000);
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.operationApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const scanWifi = async (): Promise<void> => {
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest<WifiScanResult>(
        "network.wifi.scan",
        { interface: wifiInterfaces[0]?.name },
        20_000,
      );
      if (!response.ok || !response.data) {
        throw new Error(response.error?.message ?? response.status);
      }
      setWifiScanned(true);
      setAccessPoints(response.data.access_points);
    } catch (error) {
      setWifiScanned(true);
      setAccessPoints([]);
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const connectWifi = async (): Promise<void> => {
    const wifiInterface = wifiInterfaces[0]?.name;
    if (!wifiInterface || !ssid) return;
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "network.wifi.connect",
        {
          interface: wifiInterface,
          ssid,
          psk: psk || undefined,
          allow_open: selectedAccessPoint?.security === "open",
        },
        60_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setPsk("");
      setMessage(strings.operationApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const disconnectWifi = async (): Promise<void> => {
    const wifiInterface = wifiInterfaces[0]?.name;
    if (!wifiInterface) return;
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "network.wifi.disconnect",
        { interface: wifiInterface },
        15_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.operationApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const setWifiRadio = async (action: "block" | "unblock"): Promise<void> => {
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "network.wifi.radio",
        {
          action,
          type: "wlan",
          ...(wlanRadio?.name ? { name: wlanRadio.name } : {}),
        },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.operationApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const reconnectWifi = async (): Promise<void> => {
    const wifiInterface = wifiInterfaces[0]?.name;
    if (!wifiInterface) return;
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "network.wifi.reconnect",
        { interface: wifiInterface },
        60_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.operationApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  const visibleLinks = snapshot.network.interfaces.filter((item) => item.kind !== "loopback");
  const [tab, setTab] = useState<"overview" | "wifi" | "ip">("overview");
  const connectedCount = visibleLinks.filter((item) => item.ipv4_addresses.length > 0).length;

  return (
    <HostAppShell
      ariaLabel={strings.workspaceNetwork}
      tabs={[
        {
          id: "overview",
          label: strings.hostAppTabLinks,
          count: visibleLinks.length > 0 ? visibleLinks.length : undefined,
        },
        { id: "wifi", label: strings.hostAppTabWifi },
        { id: "ip", label: strings.hostAppTabIp },
      ]}
      activeTab={tab}
      onTabChange={(id) => setTab(id as "overview" | "wifi" | "ip")}
      statusChips={[
        {
          label: associated?.ssid ?? (connectedCount > 0 ? strings.connected : strings.disconnected),
          state: associated || connectedCount > 0 ? "connected" : "disconnected",
        },
        {
          label: useDhcp ? strings.dhcp : strings.static,
          state: "local",
        },
      ]}
      message={message}
      busy={busy}
      facts={
        <>
          <HostAppFact label={strings.links} value={String(visibleLinks.length)} />
          <HostAppFact label={strings.connected} value={String(connectedCount)} />
          <HostAppFact
            label={strings.gateway}
            value={snapshot.network.default_route?.gateway ?? strings.unknown}
          />
          <HostAppFact
            label={strings.dns}
            value={snapshot.network.dns_servers[0] ?? strings.unknown}
          />
        </>
      }
    >
      {tab === "overview" ? (
        <div className="host-app-section">
        <div className="secure-interface-list">
          {visibleLinks.map((item) => {
            const association = associations.find((entry) => entry.interface === item.name);
            const wifiName = association?.ssid;
            const state =
              item.kind === "other"
                ? "other"
                : item.ipv4_addresses.length > 0
                  ? "connected"
                  : item.operstate;
            const title =
              item.kind === "wifi"
                ? wifiName ?? strings.wifi
                : item.kind === "ethernet"
                  ? strings.networkCable
                  : localizedValue(strings, item.kind);
            const linkSummary =
              formatMbps(item.speed_mbps) ??
              formatDbm(association?.signal_dbm) ??
              null;
            return (
              <article
                key={item.name}
                className="ds-link-card"
                data-active={item.name === interfaceName ? "true" : "false"}
              >
                <header>
                  <strong>{title}</strong>
                  <StatusPill
                    value={wifiName ? strings.associated : localizedValue(strings, item.kind)}
                    state={state}
                  />
                </header>
                <div>{item.ipv4_addresses.join(", ") || strings.noIpv4}</div>
                <DetailsSpoiler summary={strings.moreDetails}>
                  {linkSummary ? <p className="secure-detail">{linkSummary}</p> : null}
                  <Fact label={strings.adapterName} value={item.name} />
                  <Fact
                    label={strings.ipv6Addresses}
                    value={item.ipv6_addresses.join(", ") || strings.noIpv6}
                  />
                  <Fact
                    label={strings.hardwareAddress}
                    value={item.mac_address ?? strings.unknown}
                  />
                  <Fact
                    label={strings.interface}
                    value={localizedValue(strings, item.operstate)}
                  />
                  {formatMbps(item.speed_mbps) && (
                    <Fact label={strings.linkSpeed} value={formatMbps(item.speed_mbps)!} />
                  )}
                  {item.duplex && (
                    <Fact
                      label={strings.duplex}
                      value={
                        item.duplex === "full"
                          ? strings.fullDuplex
                          : item.duplex === "half"
                            ? strings.halfDuplex
                            : item.duplex
                      }
                    />
                  )}
                  {association?.bssid && (
                    <Fact label={strings.bssid} value={association.bssid} />
                  )}
                  {formatMhz(association?.frequency_mhz) && (
                    <Fact
                      label={strings.frequency}
                      value={formatMhz(association?.frequency_mhz)!}
                    />
                  )}
                  {formatDbm(association?.signal_dbm) && (
                    <Fact
                      label={strings.signal}
                      value={formatDbm(association?.signal_dbm)!}
                    />
                  )}
                  {formatMbps(association?.rx_mbps) && (
                    <Fact label={strings.rxRate} value={formatMbps(association?.rx_mbps)!} />
                  )}
                  {formatMbps(association?.tx_mbps) && (
                    <Fact label={strings.txRate} value={formatMbps(association?.tx_mbps)!} />
                  )}
                </DetailsSpoiler>
              </article>
            );
          })}
        </div>
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact
            label={strings.defaultRoute}
            value={snapshot.network.default_route?.interface ?? strings.unknown}
          />
          <Fact
            label={strings.gateway}
            value={snapshot.network.default_route?.gateway ?? strings.unknown}
          />
          <Fact
            label={strings.dns}
            value={snapshot.network.dns_servers.join(", ") || strings.unknown}
          />
          <Fact
            label={strings.dnsSource}
            value={localizedValue(strings, snapshot.network.dns_source)}
          />
        </DetailsSpoiler>
        </div>
      ) : null}

      {tab === "ip" ? (
        <div className="host-app-section">
        {configurable.length === 0 && <p className="secure-detail">{strings.noInterfaces}</p>}
        <SelectField
          label={strings.interface}
          value={interfaceName}
          disabled={busy}
          options={configurable.map((item) => ({
            value: item.name,
            label: linkLabel(item.kind, item.name, strings, configurable),
          }))}
          onChange={setInterfaceName}
        />
        <SettingRow label={strings.dhcpToggle}>
          <Toggle
            checked={useDhcp}
            disabled={busy || !interfaceName}
            label={strings.dhcpToggle}
            onChange={setUseDhcp}
          />
        </SettingRow>
        {!useDhcp && (
          <>
            <TextField label={strings.address} value={address} disabled={busy} onChange={setAddress} />
            <TextField label={strings.prefix} value={prefix} disabled={busy} onChange={setPrefix} />
            <TextField label={strings.gateway} value={gateway} disabled={busy} onChange={setGateway} />
          </>
        )}
        <TextField label={strings.dns} value={dns} disabled={busy} onChange={setDns} />
        <button disabled={busy || !interfaceName} onClick={() => void applyNetwork()}>
          {busy ? strings.applying : strings.apply}
        </button>
        </div>
      ) : null}

      {tab === "wifi" ? (
        <div className="host-app-section">
        <SettingRow
          label={strings.associated}
          detail={associated ? associated.ssid : strings.notAssociated}
        >
          <StatusPill
            value={associated ? associated.ssid : strings.notAssociated}
            state={associated ? "connected" : "disconnected"}
          />
        </SettingRow>
        <SettingRow
          label={strings.wifiRadios}
          detail={
            wlanRadio
              ? wlanRadio.hard_blocked
                ? strings.radioHardBlocked
                : wlanRadio.soft_blocked
                  ? strings.radioOff
                  : strings.radioOn
              : strings.unavailable
          }
        >
          <StatusPill
            value={
              wlanRadio
                ? wlanRadio.hard_blocked
                  ? strings.radioHardBlocked
                  : wlanRadio.soft_blocked
                    ? strings.radioOff
                    : strings.radioOn
                : strings.unavailable
            }
            state={
              !wlanRadio || wlanRadio.hard_blocked
                ? "degraded"
                : wlanRadio.soft_blocked
                  ? "disconnected"
                  : "connected"
            }
          />
        </SettingRow>
        <ActionBar>
          <button disabled={busy || wifiInterfaces.length === 0} onClick={() => void scanWifi()}>
            {busy ? strings.scanning : strings.scanWifi}
          </button>
          <button disabled={busy || wifiInterfaces.length === 0} onClick={() => void disconnectWifi()}>
            {strings.disconnect}
          </button>
          <button disabled={busy || wifiInterfaces.length === 0} onClick={() => void reconnectWifi()}>
            {strings.reconnectSaved}
          </button>
          <button
            disabled={busy || !radioAvailable || !wlanRadio || wlanRadio.hard_blocked}
            onClick={() => void setWifiRadio(wlanRadio?.soft_blocked ? "unblock" : "block")}
          >
            {wlanRadio?.soft_blocked ? strings.unblockRadio : strings.blockRadio}
          </button>
        </ActionBar>
        {accessPoints.length === 0 ? (
          <p className="secure-detail">
            {wifiScanned ? strings.noWifi : strings.wifiScanPrompt}
          </p>
        ) : (
          <div className="secure-access-points">
            {accessPoints.map((item) => (
              <button
                key={`${item.bssid}-${item.ssid}`}
                className="ds-choice"
                data-active={ssid === item.ssid ? "true" : "false"}
                onClick={() => setSsid(item.ssid)}
              >
                <header>
                  <strong>{item.ssid}</strong>
                  <SignalMeter percent={item.signal_percent} />
                </header>
                <span>{localizedValue(strings, item.security)}</span>
              </button>
            ))}
          </div>
        )}
        <TextField label={strings.ssid} value={ssid} disabled={busy} onChange={setSsid} />
        <TextField
          label={strings.password}
          type={showPassword ? "text" : "password"}
          value={psk}
          disabled={busy}
          onChange={setPsk}
        />
        <SettingRow label={strings.showPassword}>
          <Toggle
            checked={showPassword}
            disabled={busy}
            label={strings.showPassword}
            onChange={setShowPassword}
          />
        </SettingRow>
        <button disabled={busy || !ssid || wifiInterfaces.length === 0} onClick={() => void connectWifi()}>
          {busy ? strings.applying : strings.connect}
        </button>
        </div>
      ) : null}
      {technical ? (
        <DetailsSpoiler summary={strings.moreDetails}>
          <p className="secure-detail">{technical}</p>
        </DetailsSpoiler>
      ) : null}
      {!technical && tab === "ip" ? (
        <DetailsSpoiler summary={strings.moreDetails}>
          <p className="secure-detail">{strings.dhcpToggleHint}</p>
        </DetailsSpoiler>
      ) : null}
    </HostAppShell>
  );
}

function linkLabel(
  kind: string,
  name: string,
  strings: LocaleStrings,
  configurable: Array<{ name: string; kind: string }>,
): string {
  const friendly =
    kind === "wifi"
      ? strings.wifi
      : kind === "ethernet"
        ? strings.networkCable
        : localizedValue(strings, kind);
  const sameKind = configurable.filter((item) => item.kind === kind).length;
  return sameKind > 1 ? `${friendly} · ${name}` : friendly;
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div className="secure-fact">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
