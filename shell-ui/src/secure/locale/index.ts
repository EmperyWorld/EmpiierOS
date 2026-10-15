import { en } from "./en";
import { ru } from "./ru";

type WidenStrings<T> = {
  [K in keyof T]: T[K] extends string
    ? string
    : T[K] extends Readonly<Record<string, unknown>>
      ? WidenStrings<T[K]>
      : T[K];
};

export type LocaleStrings = WidenStrings<typeof en>;

export function localeStrings(language = import.meta.env.VITE_UI_LOCALE || "ru"): LocaleStrings {
  return language.toLocaleLowerCase().startsWith("ru") ? ru : en;
}

export function localizedValue(strings: LocaleStrings, value: string): string {
  const values = strings.values as Readonly<Record<string, string>>;
  return Object.prototype.hasOwnProperty.call(values, value) ? values[value] : value;
}
