import { es } from "../../i18n/es-MX";
import type { DocumentSummary } from "../../lib/types";
import { recordTitle } from "../profile/RecordWindow";
import type { RecordRef } from "../profile/RecordWindow";

const t = es.documents;

/** What the Expediente can be grouped by. */
export type GroupMode = "type" | "year";

/** One document of the drawer as the page draws it. */
export type DocItem = {
  id: string;
  /** the name of its document of the Expediente («Acta constitutiva»), or the file's when it has none */
  name: string;
  /** the name of the file */
  file: string;
  ext: string;
  /** its class (Legales, Financieros, Programas) */
  type: string;
  year: string;
  /** the grey line under the name */
  detail: string;
  /** the document of the Expediente it is the file of, to open its window */
  record: RecordRef | null;
};

export type Group = { key: string; items: DocItem[] };

export const defaultMode: GroupMode = "type";

export function extOf(fileName: string): string {
  const m = /\.([a-z0-9]{1,5})$/i.exec(fileName.trim());
  return m ? m[1]!.toLowerCase() : "";
}

export function fileSize(bytes: number): string {
  if (bytes >= 1048576) return `${(bytes / 1048576).toLocaleString("es-MX", { maximumFractionDigits: 1 })} MB`;
  return `${Math.max(1, Math.round(bytes / 1024))} KB`;
}

/** The kind of file of a saved document, for its tile: by its type, or by its name. */
function extOfDocument(d: DocumentSummary): string {
  const byMime: Record<string, string> = {
    "application/pdf": "pdf",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document": "docx",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet": "xlsx",
    "image/jpeg": "jpg",
    "image/png": "png",
  };
  return byMime[d.mime] ?? (extOf(d.display_name) || "txt");
}

/** The documents of the Expediente, as the backend lists them, ready to be grouped by class or by year. */
export function institutionItems(docs: DocumentSummary[]): DocItem[] {
  return docs.map((d) => {
    const type = d.category ? (t.classes[d.category] ?? t.kindOther) : t.kindOther;
    const year = d.created_at.slice(0, 4);
    const record: RecordRef | null = d.record_type ? { type: d.record_type, id: d.record_id, custom_name: d.record_name } : null;
    return {
      id: d.id,
      name: record ? recordTitle({ type: record.type, custom_name: d.record_name }) : d.display_name,
      file: d.display_name,
      ext: extOfDocument(d),
      type,
      year,
      record,
      detail: [
        d.display_name,
        t.fragments(d.chunks),
        year,
        d.redactions_count > 0 ? t.covered(d.redactions_count) : "",
        d.picture_pages > 0 ? t.fromPicture(d.picture_pages) : "",
        d.doubtful_pages.length > 0 ? t.doubtful(es.common.pageList(d.doubtful_pages)) : "",
      ]
        .filter(Boolean)
        .join(" · "),
    };
  });
}

const keyOf = (d: DocItem, mode: GroupMode) => (mode === "year" ? d.year : d.type);

/** Filters by what was typed and groups by the chosen way: years from the newest, the rest as they come. */
export function groupItems(items: DocItem[], mode: GroupMode, query: string): Group[] {
  const q = query.trim().toLowerCase();
  const shown = items.filter((d) => !q || `${d.name} ${d.file} ${d.type} ${d.year} ${d.ext}`.toLowerCase().includes(q));
  const groups: Group[] = [];
  for (const d of shown) {
    const key = keyOf(d, mode);
    const g = groups.find((x) => x.key === key);
    if (g) g.items.push(d);
    else groups.push({ key, items: [d] });
  }
  if (mode === "year") groups.sort((a, b) => Number(b.key) - Number(a.key));
  return groups;
}
