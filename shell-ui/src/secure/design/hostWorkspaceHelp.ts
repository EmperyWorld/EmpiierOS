import type { Workspace } from "../components/Taskbar";
import type { LocaleStrings } from "../locale";

type HostWorkspace = Exclude<Workspace, "eliteew">;

export function hostWorkspaceHelp(strings: LocaleStrings, workspace: HostWorkspace): string {
  switch (workspace) {
    case "system":
      return strings.hostHelpSystem;
    case "network":
      return strings.hostHelpNetwork;
    case "diagnostics":
      return strings.hostHelpDiagnostics;
    case "shield":
      return strings.hostHelpShield;
    case "browser":
      return strings.hostHelpBrowser;
    case "ota":
      return strings.hostHelpOta;
    default: {
      const _exhaustive: never = workspace;
      return _exhaustive;
    }
  }
}
