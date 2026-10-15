export type LinkornLocale = "ru" | "en";

export interface LinkornStrings {
  title: string;
  lead: string;
  commandTitle: string;
  commandBody: string;
  consoleTitle: string;
  consoleBody: string;
  browserTitle: string;
  browserBody: string;
  open: string;
  seatAbsent: string;
}

export const linkornRu: LinkornStrings = {
  title: "LinkornEW",
  lead: "Служебное кресло. Операции linkorn.* есть только в сборке профиля linkornew.",
  commandTitle: "Командный пункт",
  commandBody: "Точка входа linkornew-host command-post. Адрес только 127.0.0.1:4191, когда юнит посажен.",
  consoleTitle: "Консоль",
  consoleBody: "Точка входа linkornew-host console. Окно собирается из исходников LinkornEW. Другой tty не открывается.",
  browserTitle: "Браузер",
  browserBody: "Точка входа linkornew-host browser. Бинарник /usr/lib/station-chromium/chrome. В этот репозиторий он не входит.",
  open: "Открыть",
  seatAbsent: "Окно LinkornEW ещё не держит этот стол. Плитка ничего не запускает.",
};

export const linkornEn: LinkornStrings = {
  title: "LinkornEW",
  lead: "Staff seat. linkorn.* ops exist only in the linkornew runtime build.",
  commandTitle: "Command post",
  commandBody: "Entry linkornew-host command-post. Address is only 127.0.0.1:4191 once that unit is planted.",
  consoleTitle: "Console",
  consoleBody: "Entry linkornew-host console. The window is built from LinkornEW sources. No other tty.",
  browserTitle: "Browser",
  browserBody: "Entry linkornew-host browser. Binary /usr/lib/station-chromium/chrome. It is not in this repo.",
  open: "Open",
  seatAbsent: "The LinkornEW window is not holding this desk. The tile starts nothing.",
};
