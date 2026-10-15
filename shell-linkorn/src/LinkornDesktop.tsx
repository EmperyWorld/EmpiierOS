import { useState } from "react";
import { linkornRu, type LinkornStrings } from "./locale";
import { requestLinkorn } from "./requestLinkorn";
import { type LinkornAction } from "./seat";

const tiles: Array<{
  action: LinkornAction;
  title: keyof LinkornStrings;
  body: keyof LinkornStrings;
}> = [
  { action: "command-post", title: "commandTitle", body: "commandBody" },
  { action: "console", title: "consoleTitle", body: "consoleBody" },
  { action: "browser", title: "browserTitle", body: "browserBody" },
];

export function LinkornDesktop({ strings = linkornRu }: { strings?: LinkornStrings }) {
  const [notice, setNotice] = useState<string | null>(null);

  const open = (action: LinkornAction): void => {
    try {
      requestLinkorn(action);
      setNotice(null);
    } catch {
      setNotice(strings.seatAbsent);
    }
  };

  return (
    <main className="linkorn-desk">
      <h1>{strings.title}</h1>
      <p>{strings.lead}</p>
      {notice ? <p>{notice}</p> : null}
      <ul>
        {tiles.map((tile) => (
          <li key={tile.action}>
            <h2>{strings[tile.title]}</h2>
            <p>{strings[tile.body]}</p>
            <button type="button" onClick={() => open(tile.action)}>
              {strings.open}
            </button>
          </li>
        ))}
      </ul>
    </main>
  );
}
