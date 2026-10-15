import { useState } from "react";
import { runtimeRequest } from "../nativeTransport";
import { SeatClockSet } from "./SeatClockSet";
import { localizedValue, type LocaleStrings } from "../locale";
import { DetailsSpoiler, SelectField, TextField } from "../design/controls";
import { HostAppFact, HostAppShell } from "../design/HostAppShell";
import {
  brightnessPercent,
  fill,
  formatBytes,
  formatConnector,
  formatLockdown,
  formatOnOff,
  formatServiceUnit,
  formatUptime,
  formatYesNo,
  serviceUnitReady,
} from "../format";
import type { DomainRecord, SystemSnapshot } from "../types";

interface Props {
  snapshot: SystemSnapshot;
  strings: LocaleStrings;
  onApplied: () => Promise<void>;
}

export function SystemPanel({ snapshot, strings, onApplied }: Props) {
  const { host, power, security } = snapshot;
  const connectors = snapshot.display.data.connectors;
  const connectedDisplays = connectors.filter((item) => item.status === "connected").length;
  const backlights = snapshot.display.data.backlights ?? [];
  const brightness = backlights
    .map((item) => brightnessPercent(item.brightness, item.max_brightness))
    .find((item) => item !== null);
  const inputCount = snapshot.input.data.devices?.length ?? snapshot.input.data.event_nodes;
  const audioReady = snapshot.audio.data.cards > 0;
  const serviceUnits = snapshot.services.data.units;
  const servicesActive = serviceUnits.filter(serviceUnitReady).length;
  const rootProtected = snapshot.storage.data.root_read_only === true;
  const batterySupply = power.supplies?.find((item) => item.kind === "battery");
  const [tab, setTab] = useState<
    "overview" | "power_clock" | "devices" | "storage_services" | "protection"
  >("overview");

  return (
    <HostAppShell
      ariaLabel={strings.workspaceSystem}
      tabs={[
        { id: "overview", label: strings.hostAppTabOverview },
        { id: "power_clock", label: strings.hostAppTabPowerClock },
        { id: "devices", label: strings.hostAppTabDevices },
        { id: "storage_services", label: strings.hostAppTabStorageServices },
        { id: "protection", label: strings.hostAppTabProtection },
      ]}
      activeTab={tab}
      onTabChange={(id) =>
        setTab(id as "overview" | "power_clock" | "devices" | "storage_services" | "protection")
      }
      statusChips={[
        { label: host.hostname ?? strings.unknown, state: "local" },
        {
          label: formatLockdown(security.kernel_lockdown, strings),
          state: security.kernel_lockdown ? "connected" : "unavailable",
        },
        {
          label: rootProtected ? strings.systemDiskProtected : strings.systemDiskWritable,
          state: rootProtected ? "connected" : "degraded",
        },
      ]}
      facts={
        <>
          <HostAppFact label={strings.computerName} value={host.hostname ?? strings.unknown} />
          <HostAppFact
            label={strings.runningFor}
            value={formatUptime(snapshot.runtime.uptime_seconds, strings)}
          />
          <HostAppFact label={strings.battery} value={power.battery_present ? `${power.battery_percent ?? "—"}%` : strings.notPresent} />
          <HostAppFact label={strings.services} value={`${servicesActive}/${serviceUnits.length}`} />
        </>
      }
    >
      {tab === "overview" ? (
        <div className="host-app-section">
          <div className="host-app-tiles">
            <HostAppFact label={strings.kernel} value={host.kernel_release ?? strings.unknown} />
            <HostAppFact
              label={strings.cpu}
              value={
                host.cpu_model
                  ? host.cpu_count
                    ? `${host.cpu_model} ×${host.cpu_count}`
                    : host.cpu_model
                  : strings.unknown
              }
            />
            <HostAppFact
              label={strings.memory}
              value={
                host.mem_total_kb
                  ? `${Math.round((host.mem_available_kb ?? 0) / 1024)} / ${Math.round(host.mem_total_kb / 1024)} MB`
                  : strings.unknown
              }
            />
            <HostAppFact
              label={strings.swap}
              value={
                host.swap_total_kb == null
                  ? strings.unknown
                  : host.swap_total_kb === 0
                    ? strings.noSwap
                    : `${Math.round((host.swap_free_kb ?? 0) / 1024)} / ${Math.round(host.swap_total_kb / 1024)} MB`
              }
            />
            <HostAppFact
              label={strings.loadAverage}
              value={
                [host.loadavg_1, host.loadavg_5, host.loadavg_15]
                  .filter((item): item is string => Boolean(item))
                  .join(" ") || strings.unknown
              }
            />
            <HostAppFact
              label={strings.startCount}
              value={
                snapshot.runtime.start_count != null
                  ? String(snapshot.runtime.start_count)
                  : strings.unknown
              }
            />
          </div>
          <DetailsSpoiler summary={strings.moreDetails}>
            <Fact label={strings.bootId} value={host.boot_id ?? strings.unknown} />
            <Fact
              label={strings.uptime}
              value={`${snapshot.runtime.uptime_seconds} ${strings.seconds}`}
            />
          </DetailsSpoiler>
        </div>
      ) : null}

      {tab === "power_clock" ? (
        <div className="host-app-section">
          <div className="host-app-tiles">
            <HostAppFact
              label={strings.power}
              value={power.ac_online ? strings.pluggedIn : strings.onBattery}
            />
            {power.highest_temperature_c !== null ? (
              <HostAppFact
                label={strings.temperature}
                value={`${Math.round(power.highest_temperature_c)} °C`}
              />
            ) : null}
          </div>
        <PowerActions snapshot={snapshot} strings={strings} onApplied={onApplied} />
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact
            label={strings.powerSupplies}
            value={
              power.supplies?.length
                ? power.supplies
                    .map((item) => {
                      const state = item.state ?? (item.online == null ? null : item.online ? "online" : "offline");
                      const capacity = item.capacity != null ? `${item.capacity}%` : null;
                      return [item.name, item.kind, capacity, state].filter(Boolean).join(" ");
                    })
                    .join(", ")
                : strings.notPresent
            }
          />
          {batterySupply && (
            <>
              <Fact
                label={strings.batteryModel}
                value={
                  [batterySupply.manufacturer, batterySupply.model, batterySupply.technology]
                    .filter(Boolean)
                    .join(" ") || strings.unknown
                }
              />
              <Fact
                label={strings.chargeNow}
                value={formatMicroUnits(batterySupply.charge_now_uah, "mAh", 1000, strings)}
              />
              <Fact
                label={strings.chargeFull}
                value={formatMicroUnits(batterySupply.charge_full_uah, "mAh", 1000, strings)}
              />
              <Fact
                label={strings.designCharge}
                value={formatMicroUnits(batterySupply.charge_full_design_uah, "mAh", 1000, strings)}
              />
              <Fact
                label={strings.voltage}
                value={formatMicroUnits(batterySupply.voltage_now_uv, "V", 1_000_000, strings)}
              />
              <Fact
                label={strings.currentDraw}
                value={formatMicroUnits(batterySupply.current_now_ua, "mA", 1000, strings)}
              />
            </>
          )}
          <Fact
            label={strings.thermalZones}
            value={
              power.thermal_zones?.length
                ? power.thermal_zones
                    .map((item) =>
                      item.temperature_c != null
                        ? `${item.name} ${item.kind} ${Math.round(item.temperature_c)} °C`
                        : `${item.name} ${item.kind}`,
                    )
                    .join(", ")
                : strings.notPresent
            }
          />
        </DetailsSpoiler>
          <h3 className="host-app-section-title">{strings.clock}</h3>
        <Fact
          label={strings.timezone}
          value={snapshot.locale.data.timezone ?? strings.unknown}
        />
        <Fact
          label={strings.clockSynced}
          value={formatYesNo(snapshot.locale.data.ntp_synchronized ?? null, strings)}
        />
        <SeatClockSet strings={strings} />
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact
            label={strings.locale}
            value={snapshot.locale.data.locale ?? strings.unknown}
          />
          <Fact
            label={strings.ntpServer}
            value={snapshot.locale.data.ntp_server ?? strings.unknown}
          />
          <Fact
            label={strings.ntpLastSync}
            value={formatNtpAge(
              snapshot.locale.data.ntp_last_sync_epoch,
              snapshot.observed_at_epoch,
              strings,
            )}
          />
          <Fact
            label={strings.ntpStratum}
            value={
              snapshot.locale.data.ntp_stratum != null
                ? String(snapshot.locale.data.ntp_stratum)
                : strings.unknown
            }
          />
          <Fact
            label={strings.ntpJitter}
            value={
              snapshot.locale.data.ntp_jitter_ms != null
                ? `${snapshot.locale.data.ntp_jitter_ms} ms`
                : strings.unknown
            }
          />
          <DomainTech record={snapshot.locale} strings={strings} />
        </DetailsSpoiler>
        </div>
      ) : null}

      {tab === "devices" ? (
        <div className="host-app-section">
          <h3 className="host-app-section-title">{strings.display}</h3>
        <Fact
          label={strings.display}
          value={
            connectors.length === 0
              ? strings.displaysNone
              : fill(strings.displaysConnected, { count: connectedDisplays })
          }
        />
        {brightness !== null && brightness !== undefined && (
          <Fact label={strings.brightness} value={`${brightness}%`} />
        )}
        <BrightnessActions snapshot={snapshot} strings={strings} onApplied={onApplied} />
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact
            label={strings.gpuDriver}
            value={
              snapshot.display.data.driver
                ? snapshot.display.data.pci_id
                  ? `${snapshot.display.data.driver} ${snapshot.display.data.pci_id}`
                  : snapshot.display.data.driver
                : strings.unknown
            }
          />
          <Fact
            label={strings.connectors}
            value={
              connectors.length === 0
                ? strings.unknown
                : connectors.map(formatConnector).join(", ")
            }
          />
          <Fact
            label={strings.backlights}
            value={
              backlights.length
                ? backlights
                    .map((item) => `${item.name}: ${item.brightness ?? "—"}/${item.max_brightness ?? "—"}`)
                    .join(", ")
                : strings.notPresent
            }
          />
          <DomainTech record={snapshot.display} strings={strings} />
        </DetailsSpoiler>
          <h3 className="host-app-section-title">{strings.input}</h3>
        <Fact label={strings.input} value={fill(strings.devicesCount, { count: inputCount })} />
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact label={strings.eventNodes} value={String(snapshot.input.data.event_nodes)} />
          <Fact
            label={strings.scrollWheel}
            value={formatYesNo(snapshot.input.data.rel_wheel_observed ?? null, strings)}
          />
          <Fact
            label={strings.uinput}
            value={formatYesNo(snapshot.input.data.uinput_observed ?? null, strings)}
          />
          <Fact
            label={strings.inputDevices}
            value={
              snapshot.input.data.devices?.length
                ? snapshot.input.data.devices
                    .map((item) => {
                      const flags = [
                        item.has_key ? "key" : null,
                        item.has_rel ? "rel" : null,
                        item.has_rel_wheel ? "wheel" : null,
                        item.has_abs ? "abs" : null,
                      ]
                        .filter(Boolean)
                        .join("/");
                      return flags
                        ? `${item.node}: ${item.name} (${flags})`
                        : `${item.node}: ${item.name}`;
                    })
                    .join(", ")
                : strings.notPresent
            }
          />
          <DomainTech record={snapshot.input} strings={strings} />
        </DetailsSpoiler>
          <h3 className="host-app-section-title">{strings.sound}</h3>
        <Fact
          label={strings.sound}
          value={
            snapshot.audio.data.observed_percent != null
              ? fill(strings.volumeNow, {
                  percent: String(snapshot.audio.data.observed_percent),
                })
              : audioReady
                ? strings.ready
                : strings.notPresent
          }
        />
        <VolumeActions snapshot={snapshot} strings={strings} onApplied={onApplied} />
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact
            label={strings.playbackControl}
            value={snapshot.audio.data.playback_name ?? strings.unknown}
          />
          <Fact label={strings.audioCards} value={String(snapshot.audio.data.cards)} />
          <Fact
            label={strings.audioDevices}
            value={
              snapshot.audio.data.devices?.length
                ? snapshot.audio.data.devices
                    .map((item) => `${item.id}: ${item.name}`)
                    .join(", ")
                : strings.notPresent
            }
          />
          <DomainTech record={snapshot.audio} strings={strings} />
        </DetailsSpoiler>
        </div>
      ) : null}

      {tab === "storage_services" ? (
        <div className="host-app-section">
          <h3 className="host-app-section-title">{strings.storage}</h3>
        <Fact
          label={strings.storage}
          value={
            formatBytes(snapshot.storage.data.root_used_bytes) &&
            formatBytes(snapshot.storage.data.root_total_bytes)
              ? fill(strings.diskUsage, {
                  used: formatBytes(snapshot.storage.data.root_used_bytes)!,
                  total: formatBytes(snapshot.storage.data.root_total_bytes)!,
                })
              : rootProtected
                ? strings.systemDiskProtected
                : strings.systemDiskWritable
          }
        />
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact
            label={strings.rootFilesystem}
            value={snapshot.storage.data.root_fstype ?? strings.unknown}
          />
          {formatBytes(snapshot.storage.data.root_available_bytes) && (
            <Fact
              label={strings.diskFree}
              value={formatBytes(snapshot.storage.data.root_available_bytes)!}
            />
          )}
          <Fact
            label={strings.readOnlyRoot}
            value={formatYesNo(snapshot.storage.data.root_read_only, strings)}
          />
          <Fact
            label={strings.usbDevices}
            value={
              snapshot.usb?.data.devices?.length
                ? snapshot.usb.data.devices
                    .map((item) => {
                      const label = item.product || item.manufacturer || item.name;
                      const ids =
                        item.vendor_id && item.product_id
                          ? `${item.vendor_id}:${item.product_id}`
                          : item.name;
                      return item.speed
                        ? `${item.name} ${label} ${ids} ${item.speed}`
                        : `${item.name} ${label} ${ids}`;
                    })
                    .join(", ")
                : strings.notPresent
            }
          />
          <Fact
            label={strings.blockDevices}
            value={
              snapshot.storage.data.devices?.length
                ? snapshot.storage.data.devices
                    .map((item) =>
                      item.size_bytes != null
                        ? `${item.name}: ${Math.round(item.size_bytes / 1_000_000_000)} GB`
                        : item.name,
                    )
                    .join(", ")
                : String(snapshot.storage.data.block_devices)
            }
          />
          <DomainTech record={snapshot.storage} strings={strings} />
        </DetailsSpoiler>
          <h3 className="host-app-section-title">{strings.services}</h3>
        <Fact
          label={strings.services}
          value={
            serviceUnits.length > 0 && servicesActive === serviceUnits.length
              ? strings.servicesOk
              : fill(strings.servicesPartial, {
                  ready: servicesActive,
                  total: serviceUnits.length,
                })
          }
        />
        <DetailsSpoiler summary={strings.moreDetails}>
          {serviceUnits.map((unit) => (
            <Fact
              key={unit.name}
              label={unit.name}
              value={formatServiceUnit(unit, strings)}
            />
          ))}
          <DomainTech record={snapshot.services} strings={strings} />
        </DetailsSpoiler>
        </div>
      ) : null}

      {tab === "protection" ? (
        <div className="host-app-section">
        <div className="host-app-tiles">
          <HostAppFact label={strings.secureStartup} value={formatOnOff(security.secure_boot, strings)} />
          <HostAppFact
            label={strings.systemIntegrity}
            value={formatOnOff(security.dm_verity_root, strings)}
          />
          <HostAppFact
            label={strings.writeProtection}
            value={formatOnOff(security.root_read_only, strings)}
          />
          <HostAppFact label={strings.extraLock} value={formatLockdown(security.kernel_lockdown, strings)} />
        </div>
        <DetailsSpoiler summary={strings.moreDetails}>
          <Fact label={strings.secureBoot} value={formatYesNo(security.secure_boot, strings)} />
          <Fact label={strings.dmVerity} value={formatYesNo(security.dm_verity_root, strings)} />
          <Fact
            label={strings.lockdown}
            value={security.kernel_lockdown ?? strings.unknown}
          />
          {security.reasons.map((reason) => (
            <p key={reason} className="secure-detail">
              {localizedValue(strings, reason)}
            </p>
          ))}
          {snapshot.identity && (
            <>
              <Fact
                label={strings.kioskUser}
                value={formatYesNo(snapshot.identity.kiosk_user_observed, strings)}
              />
              <Fact
                label={strings.maintUser}
                value={formatYesNo(snapshot.identity.maint_user_observed, strings)}
              />
              <Fact
                label={strings.sshAllowUsers}
                value={
                  snapshot.identity.ssh_allow_users.length
                    ? snapshot.identity.ssh_allow_users.join(", ")
                    : strings.unknown
                }
              />
              <Fact
                label={strings.passwordLogin}
                value={formatYesNo(snapshot.identity.password_authentication, strings)}
              />
              <Fact
                label={strings.pubkeyLogin}
                value={formatYesNo(snapshot.identity.pubkey_authentication, strings)}
              />
              <Fact
                label={strings.keyboardLogin}
                value={formatYesNo(snapshot.identity.kbd_interactive_authentication, strings)}
              />
              <Fact
                label={strings.sshAuthMethods}
                value={snapshot.identity.authentication_methods ?? strings.unknown}
              />
              <Fact
                label={strings.rootLogin}
                value={formatYesNo(snapshot.identity.permit_root_login, strings)}
              />
              <Fact
                label={strings.kioskAuthorizedKeys}
                value={formatYesNo(snapshot.identity.kiosk_authorized_keys_present, strings)}
              />
              <Fact
                label={strings.maintAuthorizedKeys}
                value={formatYesNo(snapshot.identity.maint_authorized_keys_present, strings)}
              />
              <Fact
                label={strings.temporarySudoers}
                value={formatYesNo(snapshot.identity.temporary_sudoers_present, strings)}
              />
            </>
          )}
          {snapshot.watchdog && (
            <Fact
              label={strings.watchdog}
              value={
                snapshot.watchdog.present
                  ? (snapshot.watchdog.devices ?? [])
                      .map((item) => item.identity ?? item.name)
                      .join(", ") || strings.yes
                  : strings.notPresent
              }
            />
          )}
        </DetailsSpoiler>
        </div>
      ) : null}
    </HostAppShell>
  );
}

function VolumeActions({
  snapshot,
  strings,
  onApplied,
}: {
  snapshot: SystemSnapshot;
  strings: LocaleStrings;
  onApplied: () => Promise<void>;
}) {
  const available = snapshot.capabilities.some(
    (item) => item.id === "audio.control" && item.availability === "available",
  );
  const [percent, setPercent] = useState("50");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [technical, setTechnical] = useState<string | null>(null);
  const options = [10, 25, 50, 75, 100].map((value) => ({
    value: String(value),
    label: `${value}%`,
  }));

  const apply = async (): Promise<void> => {
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "audio.control",
        {
          action: "set_volume",
          percent: Number.parseInt(percent, 10),
          name: "Speaker",
        },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.volumeApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="secure-power-actions">
      <SelectField
        label={strings.volume}
        value={percent}
        disabled={!available || busy}
        options={options}
        onChange={setPercent}
      />
      <div className="secure-inline-actions">
        <button type="button" disabled={!available || busy} onClick={() => void apply()}>
          {strings.applyVolume}
        </button>
      </div>
      {message && <p className="secure-detail">{message}</p>}
      <DetailsSpoiler summary={strings.moreDetails}>
        <p className="secure-detail">{strings.volumeHint}</p>
        {!available && (
          <p className="secure-detail">{localizedValue(strings, "audio_helper_not_installed")}</p>
        )}
        {technical && <p className="secure-detail">{technical}</p>}
      </DetailsSpoiler>
    </div>
  );
}

function BrightnessActions({
  snapshot,
  strings,
  onApplied,
}: {
  snapshot: SystemSnapshot;
  strings: LocaleStrings;
  onApplied: () => Promise<void>;
}) {
  const available = snapshot.capabilities.some(
    (item) => item.id === "display.control" && item.availability === "available",
  );
  const backlights = snapshot.display.data.backlights ?? [];
  const current = backlights
    .map((item) => brightnessPercent(item.brightness, item.max_brightness))
    .find((item) => item !== null);
  const [percent, setPercent] = useState(String(current ?? 100));
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [technical, setTechnical] = useState<string | null>(null);
  const options = [10, 25, 50, 75, 100].map((value) => ({
    value: String(value),
    label: `${value}%`,
  }));

  const apply = async (): Promise<void> => {
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "display.control",
        {
          action: "set_brightness",
          percent: Number.parseInt(percent, 10),
          ...(backlights[0]?.name ? { name: backlights[0].name } : {}),
        },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setMessage(strings.brightnessApplied);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="secure-power-actions">
      <SelectField
        label={strings.brightness}
        value={percent}
        disabled={!available || busy}
        options={options}
        onChange={setPercent}
      />
      <div className="secure-inline-actions">
        <button type="button" disabled={!available || busy} onClick={() => void apply()}>
          {strings.applyBrightness}
        </button>
      </div>
      {message && <p className="secure-detail">{message}</p>}
      <DetailsSpoiler summary={strings.moreDetails}>
        <p className="secure-detail">{strings.brightnessHint}</p>
        {!available && (
          <p className="secure-detail">
            {localizedValue(strings, "backlight_helper_not_installed")}
          </p>
        )}
        {technical && <p className="secure-detail">{technical}</p>}
      </DetailsSpoiler>
    </div>
  );
}

function PowerActions({
  snapshot,
  strings,
  onApplied,
}: {
  snapshot: SystemSnapshot;
  strings: LocaleStrings;
  onApplied: () => Promise<void>;
}) {
  const available = snapshot.capabilities.some(
    (item) => item.id === "system.power" && item.availability === "available",
  );
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [technical, setTechnical] = useState<string | null>(null);

  const run = async (action: "reboot" | "poweroff"): Promise<void> => {
    const expected = action === "reboot" ? "REBOOT" : "POWEROFF";
    if (confirm !== expected) {
      setMessage(strings.powerConfirmMismatch);
      setTechnical(strings.powerConfirmHint);
      return;
    }
    setBusy(true);
    setMessage(null);
    setTechnical(null);
    try {
      const response = await runtimeRequest(
        "system.power",
        { action, confirm: expected },
        8_000,
      );
      if (!response.ok) {
        throw new Error(response.error?.message ?? response.status);
      }
      setConfirm("");
      setMessage(strings.powerScheduled);
      await onApplied();
    } catch (error) {
      setMessage(strings.operationFailed);
      setTechnical(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="secure-power-actions">
      <TextField
        label={strings.powerConfirmLabel}
        value={confirm}
        disabled={!available || busy}
        onChange={setConfirm}
      />
      <div className="secure-inline-actions">
        <button type="button" disabled={!available || busy} onClick={() => void run("reboot")}>
          {strings.reboot}
        </button>
        <button type="button" disabled={!available || busy} onClick={() => void run("poweroff")}>
          {strings.poweroff}
        </button>
      </div>
      {message && <p className="secure-detail">{message}</p>}
      <DetailsSpoiler summary={strings.moreDetails}>
        <p className="secure-detail">{strings.powerHintSimple}</p>
        <p className="secure-detail">{strings.powerConfirmHint}</p>
        <Fact label={strings.source} value={snapshot.power.source} />
        <Fact label={strings.ac} value={formatYesNo(snapshot.power.ac_online, strings)} />
        <Fact
          label={strings.battery}
          value={
            snapshot.power.battery_state
              ? localizedValue(strings, snapshot.power.battery_state)
              : strings.unknown
          }
        />
        {!available && (
          <p className="secure-detail">{localizedValue(strings, "power_units_not_installed")}</p>
        )}
        {technical && <p className="secure-detail">{technical}</p>}
      </DetailsSpoiler>
    </div>
  );
}

function formatMicroUnits(
  value: number | null | undefined,
  unit: string,
  divisor: number,
  strings: LocaleStrings,
): string {
  if (value == null || divisor <= 0) {
    return strings.unknown;
  }
  const scaled = value / divisor;
  const text = scaled >= 10 ? String(Math.round(scaled)) : scaled.toFixed(2);
  return `${text} ${unit}`;
}

function formatNtpAge(
  lastEpoch: number | null | undefined,
  observedEpoch: number,
  strings: LocaleStrings,
): string {
  if (lastEpoch == null) {
    return strings.unknown;
  }
  const age = observedEpoch - lastEpoch;
  if (age < 0 || age > 30 * 24 * 60 * 60) {
    return String(lastEpoch);
  }
  return fill(strings.ntpLastSyncAgo, { seconds: String(age) });
}

function DomainTech({
  record,
  strings,
}: {
  record: DomainRecord<unknown>;
  strings: LocaleStrings;
}) {
  return (
    <>
      <Fact label={strings.source} value={record.source} />
      {record.reason && (
        <p className="secure-detail">
          {localizedValue(strings, record.availability)} · {localizedValue(strings, record.reason)}
        </p>
      )}
    </>
  );
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div className="secure-fact">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
