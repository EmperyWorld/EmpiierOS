import { parseNotificationList, toastEligible } from "./notificationIpc";

function assert(condition: unknown, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

const list = parseNotificationList({
  notifications: [
    { id: "n1", title: "a", message: "m", severity: "info", read: false, suppressed: false },
    { id: "n2", title: "b", message: "m", severity: "warning", read: false, suppressed: true },
    { id: "n3", title: "c", message: "m", severity: "critical", read: false, suppressed: false },
    { id: "n4", title: "d", message: "m", severity: "info", read: true, suppressed: false },
    { id: "n5", title: "e", message: "m", severity: "bogus", read: false },
  ],
  unread_count: 4,
  critical_count: 1,
  kiosk_quiet_mode: true,
});

assert(list.quietMode, "quiet mode parsed");
assert(list.unreadCount === 4 && list.criticalCount === 1, "counts parsed");
assert(list.items[4].severity === "info", "unknown severity falls back to info");
assert(list.items[4].suppressed === false, "missing suppressed is false");
assert(
  list.items.filter(toastEligible).map((item) => item.id).join(",") === "n1,n3,n5",
  "suppressed and read items never toast",
);
assert(!parseNotificationList({}).quietMode, "missing quiet flag is off");

console.log("notificationIpc.check ok");
