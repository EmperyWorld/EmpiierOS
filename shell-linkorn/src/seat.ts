export const LINKORN_ACTIONS = ["command-post", "console", "browser"] as const;

export type LinkornAction = (typeof LINKORN_ACTIONS)[number];

export function isLinkornAction(word: string): word is LinkornAction {
  return (LINKORN_ACTIONS as readonly string[]).includes(word);
}
