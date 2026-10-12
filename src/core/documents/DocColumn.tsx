import { Icon } from "../../components/icons";
import { AddSlot, Avatar, Button, FileTile, Folder, IconButton, ListRow, Search, Segmented, Subfolder, Tag, seriesTone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { groupItems, targetOf } from "./documentsModel";
import type { DocCol, DocItem, GroupMode, UploadTarget } from "./documentsModel";
import type { Passage } from "../../lib/types";

/** What the search found inside the text of the documents (IA5): only in «De mi institución». */
export type Inside = { loading: boolean; passages: Passage[] };

const t = es.documents;

/** One container of the page: «De mi institución» or «De los donantes», with its search, its grouping and its subfolders. */
export function DocColumn({
  col,
  items,
  query,
  onQuery,
  mode,
  onMode,
  open,
  onToggle,
  onUpload,
  onDropFile,
  asking,
  onAsk,
  onRemove,
  busy,
  inside,
  onType,
}: {
  col: DocCol;
  items: DocItem[];
  query: string;
  onQuery: (q: string) => void;
  mode: GroupMode;
  onMode: (m: GroupMode) => void;
  /** what the person opened or closed by hand; what was never touched follows the default */
  open: Record<string, boolean>;
  onToggle: (id: string, now: boolean) => void;
  /** with a target the destination is known; without one the form asks for it */
  onUpload: (target: UploadTarget | null, scope?: DocCol) => void;
  onDropFile: (target: UploadTarget, file: File) => void;
  asking: string | null;
  onAsk: (id: string | null) => void;
  onRemove: (id: string) => void;
  busy: boolean;
  /** the passages found inside the documents, while the person searches */
  inside?: Inside | null;
  /** to say what a document is (IA6): only the documents of the institution */
  onType?: (d: DocItem) => void;
}) {
  const c = t.cols[col];
  const groups = groupItems(items, mode, query);
  const searching = query.trim() !== "";

  return (
    <Folder tone="ac" title={c.title} chip={<span className="folder-count tabular">{items.length}</span>} bodyClassName="!gap-4">
      <p className="text-ui text-ink-2">{c.note}</p>

      {items.length === 0 ? (
        <div className="space-y-3">
          <p className="rounded-inset bg-inset px-5 py-4 text-ui font-semibold text-ink-2">{c.empty}</p>
          <AddSlot onClick={() => onUpload(null, col)}>{c.add}</AddSlot>
        </div>
      ) : (
        <>
          <div className="flex flex-wrap items-center gap-3">
            <Search label={c.search} placeholder={c.search} value={query} onChange={(e) => onQuery(e.target.value)} className="min-w-[200px] flex-1" />
            <Segmented
              label={t.groupBy}
              prefix={t.groupBy}
              value={mode}
              onChange={onMode}
              items={c.groups.map(([id, label]) => ({ id: id as GroupMode, label }))}
            />
          </div>

          {groups.length === 0 ? (
            !inside || (!inside.loading && inside.passages.length === 0) ? <p className="px-2 py-4 text-ui text-ink-3">{t.noMatch}</p> : null
          ) : (
            <div className="subfolders">
              {groups.map((g, i) => {
                const id = `${col}:${mode}:${g.key}`;
                const isOpen = open[id] ?? (searching ? true : i === 0);
                const target = targetOf(col, mode, g.key);
                return (
                  <Subfolder
                    key={id}
                    name={g.key}
                    count={g.items.length}
                    lead={
                      mode === "donor" ? (
                        <Avatar name={g.key} size="sm" tone={seriesTone(i + 1)} />
                      ) : (
                        <span className="flex text-ink-2">
                          <Icon name="folder" size={18} />
                        </span>
                      )
                    }
                    open={isOpen}
                    onToggle={() => onToggle(id, !isOpen)}
                    uploadLabel={t.uploadTo(g.key)}
                    dropHint={t.dropHint}
                    onUpload={() => onUpload(target)}
                    onDropFile={(file) => onDropFile(target, file)}
                  >
                    {g.items.map((d) => (
                      <DocRow key={d.id} d={d} asking={asking === d.id} onAsk={onAsk} onRemove={onRemove} onType={onType} busy={busy} />
                    ))}
                  </Subfolder>
                );
              })}
            </div>
          )}

          {searching && inside && (
            <section aria-label={t.inside} className="space-y-1">
              <h3 className="px-2 text-ui font-bold text-ink-2">{t.inside}</h3>
              {inside.loading ? (
                <p className="px-2 py-2 text-ui text-ink-3">{t.insideSearching}</p>
              ) : inside.passages.length === 0 ? (
                <p className="px-2 py-2 text-ui text-ink-3">{t.insideNone}</p>
              ) : (
                <ul>
                  {inside.passages.map((p) => (
                    <PassageRow key={p.chunk} p={p} ext={items.find((d) => d.id === p.document_id)?.ext ?? "txt"} />
                  ))}
                </ul>
              )}
            </section>
          )}
        </>
      )}
    </Folder>
  );
}

/** A piece of a document that answers the search: the document, its page and the words around what was looked for. */
function PassageRow({ p, ext }: { p: Passage; ext: string }) {
  return (
    <li className="list-row !items-start">
      <FileTile ext={ext} />
      <div className="min-w-0 flex-1">
        <b className="block truncate font-bold">{p.document}</b>
        {p.page !== null && <small className="block text-small font-medium text-ink-3">{t.page(p.page)}</small>}
        <p className="mt-1 line-clamp-3 text-ui text-ink-2">{p.snippet}</p>
      </div>
    </li>
  );
}

function DocRow({
  d,
  asking,
  onAsk,
  onRemove,
  onType,
  busy,
}: {
  d: DocItem;
  asking: boolean;
  onAsk: (id: string | null) => void;
  onRemove: (id: string) => void;
  onType?: (d: DocItem) => void;
  busy: boolean;
}) {
  return (
    <ListRow
      lead={<FileTile ext={d.ext} />}
      title={d.name}
      detail={d.detail}
      state={d.reading ? <Tag tone="amber">{t.reading}</Tag> : <Tag tone="green" variant="soft" className="max-sm:hidden">{t.read}</Tag>}
      actions={
        d.reading ? undefined : asking ? (
          <span className="inline-flex items-center gap-1">
            <Button size="sm" variant="danger" title={t.removeHelp} disabled={busy} autoFocus onClick={() => onRemove(d.id)}>
              {es.common.remove}
            </Button>
            <Button size="sm" variant="plain" onClick={() => onAsk(null)}>
              {es.common.cancel}
            </Button>
          </span>
        ) : (
          <span className="inline-flex items-center gap-1">
            {onType && <IconButton icon="pencil" label={t.changeType(d.name)} variant="plain" size="sm" onClick={() => onType(d)} />}
            <IconButton icon="trash" label={t.removeDoc(d.name)} variant="plain" size="sm" onClick={() => onAsk(d.id)} />
          </span>
        )
      }
      className={asking ? "!bg-inset" : ""}
    />
  );
}
