import {
  isSeatFloorModule,
  visibleHostModules,
} from "./foflineHostModules";
import {
  LAUNCHER_JOURNEY,
  journeyFocus,
  journeyMark,
  launcherStage,
  mayAutoBringUpPortal,
  notePortalBroughtUp,
  seatLicenseReady,
  type LauncherStage,
} from "./foflineLauncherStage";

function assert(condition: unknown, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

assert(
  LAUNCHER_JOURNEY.join(">") === "pin>consume>portal>spa>login>ready",
  "journey order drifted",
);

assert(launcherStage(null) === "offline", "null status is offline");
assert(mayAutoBringUpPortal("portal"), "portal may one-shot channel_up");
assert(!mayAutoBringUpPortal("ready"), "ready must not auto-bring portal");
assert(!mayAutoBringUpPortal("login"), "login must not auto-bring portal");

assert(journeyFocus("offline") === null, "offline has no focus");
assert(journeyFocus("pin_missing") === "pin", "missing PIN focuses unlock");
assert(journeyFocus("pin_locked") === "pin", "lockout focuses unlock");
assert(journeyFocus("channel_invalid") === "consume", "bad channel focuses activate");
assert(journeyFocus("ready") === "ready", "ready focuses open");

const cases: Array<[LauncherStage, string]> = [
  ["offline", "ahead,ahead,ahead,ahead,ahead,ahead"],
  ["pin", "current,ahead,ahead,ahead,ahead,ahead"],
  ["pin_missing", "blocked,ahead,ahead,ahead,ahead,ahead"],
  ["pin_locked", "blocked,ahead,ahead,ahead,ahead,ahead"],
  ["consume", "done,current,ahead,ahead,ahead,ahead"],
  ["channel_invalid", "done,blocked,ahead,ahead,ahead,ahead"],
  ["portal", "done,done,current,ahead,ahead,ahead"],
  ["spa", "done,done,done,current,ahead,ahead"],
  ["login", "done,done,done,done,current,ahead"],
  ["ready", "done,done,done,done,done,current"],
];

for (const [stage, expected] of cases) {
  const marks = LAUNCHER_JOURNEY.map((step) => journeyMark(step, stage)).join(",");
  assert(marks === expected, `${stage} marks ${marks} != ${expected}`);
}

assert(isSeatFloorModule("launcher"), "launcher is seat floor");
assert(isSeatFloorModule("network"), "network is seat floor");
assert(!isSeatFloorModule("browser"), "browser waits for license");
assert(!isSeatFloorModule("ota"), "ota waits for license");
assert(
  visibleHostModules(false).every((item) => item.id === "launcher" || item.id === "network"),
  "unlicensed glass is launcher + network",
);
assert(visibleHostModules(true).length > 2, "licensed glass shows product tiles");

assert(!seatLicenseReady(null), "no status is unlicensed");
assert(
  !seatLicenseReady({ pin_issued: true, pin_unlocked: true, capsule_imported: false }),
  "PIN-only is not a license",
);
assert(
  seatLicenseReady({
    pin_issued: true,
    pin_unlocked: true,
    capsule_imported: true,
    entitlement_present: true,
  }),
  "consume is the license",
);

assert(launcherStage({ pin_issued: false, capsule_imported: false }) === "pin_missing", "pin_missing");
assert(
  launcherStage({ pin_issued: true, pin_unlocked: false, capsule_imported: false }) === "pin",
  "pin",
);
assert(
  launcherStage({
    pin_issued: true,
    pin_unlocked: true,
    capsule_imported: true,
    entitlement_present: true,
    channel_present: true,
    channel_valid: true,
    channel_up: true,
    spa_state: "ready",
    oidc_session: "present",
  }) === "ready",
  "ready",
);

assert(launcherStage(notePortalBroughtUp({
  pin_issued: true,
  pin_unlocked: true,
  capsule_imported: true,
  entitlement_present: true,
  channel_present: true,
  channel_valid: true,
  channel_up: false,
  spa_state: "missing",
}, "applied")) === "spa", "portal result leaves the portal step");
assert(
  notePortalBroughtUp({
    pin_issued: true,
    pin_unlocked: true,
    capsule_imported: true,
    channel_up: false,
    portal_hostmap: "missing",
  }, "applied").portal_hostmap === "applied",
  "portal result marks host-map applied",
);

console.log("foflineLauncherStage.check ok");
