import { useState } from "react";
import { localizedValue, type LocaleStrings } from "../locale";
import { DetailsSpoiler, StatusPill } from "../design/controls";
import { HostAppFact, HostAppShell } from "../design/HostAppShell";
import type { RuntimeEvent, SystemSnapshot } from "../types";

interface Props {
  snapshot: SystemSnapshot;
  events: RuntimeEvent[];
  strings: LocaleStrings;
}

type DiagTab = "overview" | "events" | "capabilities";

export function DiagnosticsPanel({ snapshot, events, strings }: Props) {
  const [tab, setTab] = useState<DiagTab>("overview");
  const availableCaps = snapshot.capabilities.filter(
    (item) => item.availability === "available",
  ).length;

  return (
    <HostAppShell
      ariaLabel={strings.workspaceDiagnostics}
      tabs={[
        { id: "overview", label: strings.hostAppTabOverview },
        {
          id: "events",
          label: strings.hostAppTabEvents,
          count: events.length > 0 ? events.length : undefined,
        },
        {
          id: "capabilities",
          label: strings.hostAppTabCapabilities,
          count: availableCaps > 0 ? availableCaps : undefined,
        },
      ]}
      activeTab={tab}
      onTabChange={(id) => setTab(id as DiagTab)}
      statusChips={[
        {
          label:
            snapshot.overall_status === "verified" ? strings.ok : strings.attention,
          state: snapshot.overall_status === "verified" ? "connected" : "degraded",
        },
        {
          label: strings.events,
          state: events.length > 0 ? "observed" : "local",
        },
      ]}
      facts={
        <>
          <HostAppFact
            label={strings.runtime}
            value={localizedValue(strings, snapshot.runtime.status)}
          />
          <HostAppFact label={strings.sequence} value={String(snapshot.sequence)} />
          <HostAppFact label={strings.events} value={String(events.length)} />
          <HostAppFact label={strings.capabilities} value={String(snapshot.capabilities.length)} />
        </>
      }
    >
      {tab === "overview" ? (
        <div className="host-app-section">
          <div className="host-app-tiles">
            <HostAppFact label={strings.source} value={snapshot.network.source} />
            <HostAppFact label={strings.securityPosture} value={snapshot.security.source} />
          </div>
        </div>
      ) : null}

      {tab === "events" ? (
        events.length === 0 ? (
          <p className="host-app-empty">{strings.eventsEmpty}</p>
        ) : (
          <ol className="secure-events">
            {events.map((event) => (
              <li key={`${event.sequence}-${event.domain}-${event.kind}`}>
                <strong>{event.summary}</strong>
                <DetailsSpoiler summary={strings.moreDetails}>
                  <span>
                    #{event.sequence} · {event.domain} · {localizedValue(strings, event.kind)}
                  </span>
                </DetailsSpoiler>
              </li>
            ))}
          </ol>
        )
      ) : null}

      {tab === "capabilities" ? (
        <div className="secure-capabilities">
          {snapshot.capabilities.map((capability) => (
            <article
              key={capability.id}
              className="secure-capability"
              data-state={capability.availability}
            >
              <div>
                <strong>{localizedValue(strings, capability.id)}</strong>
                <StatusPill
                  value={localizedValue(strings, capability.availability)}
                  state={capability.availability}
                />
              </div>
              <DetailsSpoiler summary={strings.moreDetails}>
                <p>
                  {capability.reason
                    ? localizedValue(strings, capability.reason)
                    : strings.disabledReason}
                </p>
              </DetailsSpoiler>
            </article>
          ))}
        </div>
      ) : null}
    </HostAppShell>
  );
}
