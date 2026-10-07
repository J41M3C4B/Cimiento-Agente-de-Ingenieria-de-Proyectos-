import { es } from "../../i18n/es-MX";
import type { DocumentSummary } from "../../lib/types";

const t = es.documents;

/** The two containers of the page. */
export type DocCol = "inst" | "donor";
/** What a container can be grouped by. */
export type GroupMode = "type" | "year" | "donor";

/** One document as the page draws it (a real one from the backend, or a pending one while it is being read). */
export type DocItem = {
  id: string;
  /** the name as the person wrote it (it may already carry its extension) */
  name: string;
  ext: string;
  type: string;
  year: string;
  donor?: string;
  /** the grey line under the name */
  detail: string;
  /** still being read: it shows «Leyendo…» and has no actions */
  reading?: boolean;
};

export type Group = { key: string; items: DocItem[] };

/** Where a file goes when it is dropped on, or uploaded from, a subfolder: the destination is already known. */
export type UploadTarget = { col: DocCol; label: string; donor?: string; type?: string; year?: string };

/** What the upload form hands over. */
export type Draft = {
  scope: DocCol;
  name: string;
  text: string;
  donor: string;
  kind: string;
  file: { name: string; size: string; ext: string } | null;
  target: UploadTarget | null;
};

export const defaultMode = (col: DocCol): GroupMode => (col === "inst" ? "type" : "donor");

export function extOf(fileName: string): string {
  const m = /\.([a-z0-9]{1,5})$/i.exec(fileName.trim());
  return m ? m[1]!.toLowerCase() : "";
}

export const baseName = (fileName: string) => fileName.replace(/\.[a-z0-9]{1,5}$/i, "");

export function fileSize(bytes: number): string {
  if (bytes >= 1048576) return `${(bytes / 1048576).toLocaleString("es-MX", { maximumFractionDigits: 1 })} MB`;
  return `${Math.max(1, Math.round(bytes / 1024))} KB`;
}

/** Plain text files can be read in the browser; the rest (PDF, Word, Excel…) cannot be read here yet. */
export function isPlainText(file: File): boolean {
  return ["txt", "md", "csv"].includes(extOf(file.name)) || file.type.startsWith("text/");
}

const kindLabel = (kind: string) => t.kinds[kind] ?? t.kindOther;

/** The documents of the institution, as the backend lists them, ready to be grouped by type or by year. */
export function institutionItems(docs: DocumentSummary[]): DocItem[] {
  return docs.map((d) => {
    const type = kindLabel(d.kind);
    const year = d.created_at.slice(0, 4);
    return {
      id: d.id,
      name: d.display_name,
      ext: extOf(d.display_name) || "txt",
      type,
      year,
      detail: [type, t.fragments(d.chunks), year, d.redactions_count > 0 ? t.covered(d.redactions_count) : ""].filter(Boolean).join(" · "),
    };
  });
}

/**
 * TODO(logica): the documents of donors have no support in the backend yet (no table, no command). When they exist,
 * return them here as `DocItem`s with `donor`, `type` («Reglas», «Indicadores», «Guías y formatos») and
 * `detail` («Reglas · 1.4 MB · Usado en 2 convocatorias», see `t.usedIn`). Until then the container shows its
 * friendly empty state.
 */
export function donorItems(): DocItem[] {
  return [];
}

/**
 * TODO(logica): save a document of a donor. Needs a backend command that takes the text, the donor and the type
 * (something like `document_add_text(…, scope: "donor", donor, kind)`) and that `documents_list` returns the donor and
 * the kind. Until then the form says so and nothing is saved.
 */
export async function addDonorDocument(_draft: Draft): Promise<{ status: "not_connected" }> {
  return { status: "not_connected" };
}

const keyOf = (d: DocItem, mode: GroupMode) => (mode === "donor" ? d.donor ?? t.donorUnassigned : mode === "year" ? d.year : d.type);

/** Filters by what was typed and groups by the chosen way: years from the newest, the rest as they come. */
export function groupItems(items: DocItem[], mode: GroupMode, query: string): Group[] {
  const q = query.trim().toLowerCase();
  const shown = items.filter((d) => !q || `${d.name} ${d.type} ${d.donor ?? ""} ${d.year} ${d.ext}`.toLowerCase().includes(q));
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

/** The destination of a subfolder, according to the way its container is grouped. */
export function targetOf(col: DocCol, mode: GroupMode, key: string): UploadTarget {
  const target: UploadTarget = { col, label: key };
  if (mode === "donor") target.donor = key;
  else if (mode === "year") target.year = key;
  else target.type = key;
  return target;
}
