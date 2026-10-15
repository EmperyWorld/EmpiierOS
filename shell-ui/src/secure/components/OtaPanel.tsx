import type { LocaleStrings } from "../locale";
import { SeatOtaSection } from "./SeatOtaSection";

type Props = {
  strings: LocaleStrings;
};

export function OtaPanel({ strings }: Props) {
  return <SeatOtaSection strings={strings} />;
}
