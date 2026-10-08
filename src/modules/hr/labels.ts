// Words for the codes of the staff module.
import { es } from "../../i18n/es-MX";
import type { ModalityInfo } from "./types";

/** The name of a modality: the institution's own title, or the words of a built-in one. */
export const modalityName = (m: ModalityInfo) => m.title ?? es.hr.modalities[m.code] ?? m.code;

/** The name of a modality code, looking it up among the known ones. */
export const modalityOf = (code: string, all: ModalityInfo[]) => {
  const m = all.find((x) => x.code === code);
  return m ? modalityName(m) : (es.hr.modalities[code] ?? code);
};
