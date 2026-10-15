import {
  mapSeatBrowserStatus,
  normalizeAllowHosts,
  normalizePolicyHost,
  parsePolicyHostList,
  parseDownloadList,
  parseTabList,
} from "./seatBrowser";

function assert(condition: unknown, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

assert(normalizePolicyHost("  App.EliteEW.ru ") === "app.eliteew.ru", "trim+lower host");
assert(normalizePolicyHost("https://meet.eliteew.ru/room") === "meet.eliteew.ru", "url host");
assert(normalizePolicyHost("user@host") === null, "reject userinfo");
assert(normalizePolicyHost("a b.com") === null, "reject space");
assert(normalizePolicyHost("10.0.0.0/8") === "10.0.0.0/8", "keep cidr");

assert(
  normalizeAllowHosts(["App.eliteew.ru", "app.eliteew.ru", " ", "auth.eliteew.ru"]).join(",") ===
    "app.eliteew.ru,auth.eliteew.ru",
  "unique allow hosts",
);

assert(parsePolicyHostList("ya.ru") === undefined, "non-array is unobserved");
{
  const hosts = parsePolicyHostList(["ya.ru", " ya.ru ", ""]);
  assert(hosts !== undefined && hosts.join(",") === "ya.ru", "dedupe observed");
}

const smeared = mapSeatBrowserStatus({
  allow_count: 3,
  policy_mode: "allowlist",
});
assert(smeared.allowHostsObserved === false, "counts-only status is unobserved");
assert(smeared.allowHosts.length === 0, "unobserved list stays empty");

const living = mapSeatBrowserStatus({
  allow: ["ya.ru", "app.eliteew.ru"],
  allow_count: 2,
  deny: [],
  policy_mode: "allowlist",
  is_default: false,
});
assert(living.allowHostsObserved, "array marks observed");
assert(living.allowHosts.join("\n") === "ya.ru\napp.eliteew.ru", "living hosts stay one-per-row");
assert(living.allowCount === 2, "count follows payload");
assert(living.camera === true, "missing camera defaults on");
assert(living.microphone === true, "missing microphone defaults on");

const mediaOff = mapSeatBrowserStatus({
  allow: [],
  camera: false,
  microphone: false,
});
assert(mediaOff.camera === false, "camera false is observed");
assert(mediaOff.microphone === false, "microphone false is observed");

assert(parseTabList("nope").length === 0, "non-array tabs empty");
{
  const tabs = parseTabList([
    { id: 1, host: "app.eliteew.ru", url: "https://app.eliteew.ru/" },
    { id: 1, host: "dup", url: "https://dup/" },
    { id: 2, host: "meet.eliteew.ru", url: "https://meet.eliteew.ru/" },
    { id: 3, host: "auth.eliteew.ru", url: "https://auth.eliteew.ru/" },
    { id: 4, host: "overflow", url: "https://overflow/" },
  ]);
  assert(tabs.length === 3, "tab cap 3");
  assert(tabs.map((tab) => tab.id).join(",") === "1,2,3", "unique tab ids");
}

const withTabs = mapSeatBrowserStatus({
  allow: [],
  tab_count: 2,
  tab_cap: 3,
  active_tab: 2,
  tabs: [{ id: 2, host: "app.eliteew.ru", url: "https://app.eliteew.ru/" }],
});
assert(withTabs.tabCount === 2, "tab count from status");
assert(withTabs.tabCap === 3, "tab cap");
assert(withTabs.activeTab === 2, "active tab");
assert(withTabs.tabs[0]?.host === "app.eliteew.ru", "tab host mapped");

assert(parseDownloadList("nope").length === 0, "non-array downloads empty");
{
  const downloads = parseDownloadList([
    { id: 1, name: "a.pdf", host: "app.eliteew.ru", bytes: 12, phase: "finished" },
    { id: 1, name: "dup", host: "x", bytes: 1, phase: "finished" },
    { id: 2, name: "../etc/passwd", host: "x", bytes: 1, phase: "finished" },
  ]);
  assert(downloads.length === 1, "download basename only");
  assert(downloads[0]?.name === "a.pdf", "download name mapped");
}

const withDl = mapSeatBrowserStatus({
  allow: [],
  downloads: [{ id: 3, name: "note.txt", host: "app.eliteew.ru", bytes: 4, phase: "finished" }],
  download_count: 1,
});
assert(withDl.downloadCount === 1, "download count");
assert(withDl.downloads[0]?.phase === "finished", "download phase");

console.log("seatBrowser.check ok");
