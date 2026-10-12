import { Icon } from "../../components/icons";
import { AddSlot, Button, FileTile, Folder, IconButton, ListRow, Search, Segmented, Subfolder, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { groupItems } from "./documentsModel";
import type { DocItem, GroupMode } from "./documentsModel";
import type { Passage } from "../../lib/types";

const t = es.documents;

/** What the search found inside the text of the documents of the drawer (IA5). */
export type Inside = { loading: boolean; passages: Passage[] };

/**
 * The drawer of the Expediente in the tray (ADR-036): its documents grouped by class or by year, its search (by name,
 * and inside their text), and for each one its window and «Borrar esto y todo lo que salió de ahí».
 */
export function DocColumn({
  items,
  query,
  onQuery,
  mode,
  onMode,
  open,
  onToggle,
  onUpload,
  onOpen,
  asking,
  onAsk,
  onRemove,
  busy,
  inside,
}: {
  items: DocItem[];
  query: string;
  onQuery: (q: string) => void;
  mode: GroupMode;
  onMode: (m: GroupMode) => void;
  /** what the person opened or closed by hand; what was never touched follows the default */
  open: Record<string, boolean>;
  onToggle: (id: string, now: boolean) => void;
  onUpload: () => void;
  /** the window of its document of the Expediente */
  onOpen: (d: DocItem) => void;
  asking: string | null;
  onAsk: (id: string | null) => void;
  onRemove: (id: string) => void;
  busy: boolean;
  /** the passages found inside the documents, while the person searches */
  inside?: Inside | null;
}) {
  const c = t.cols.inst;
  const groups = groupItems(items, mode, query);
  const searching = query.trim() !== "";

  return (
    <Folder tone="ac" title={c.title} chip={<span className="folder-count tabular">{items.length}</span>} bodyClassName="!gap-4">
      <p className="text-ui text-ink-2">{c.note}</p>

      {items.length === 0 ? (
        <div className="space-y-3">
          <p className="rounded-inset bg-inset px-5 py-4 text-ui font-semibold text-ink-2">{c.empty}</p>
          <AddSlot onClick={onUpload}>{c.add}</AddSlot>
        </div>
      ) : (
        <>
          <div className="flex flex-wrap items-center gap-3">
            <Search label={c.search} placeholder={c.search} value={query} onChange={(e) => onQuery(e.target.value)} className="min-w-[200px] flex-1" />
            <Segmented label={t.groupBy} prefix={t.groupBy} value={mode} onChange={onMode} items={c.groups.map(([id, label]) => ({ id: id as GroupMode, label }))} />
          </div>

          {groups.length === 0 ? (
            !inside || (!inside.loading && inside.passages.length === 0) ? <p className="px-2 py-4 text-ui text-ink-3">{t.noMatch}</p> : null
          ) : (
            <div className="subfolders">
              {groups.map((g, i) => {
                const id = `record:${mode}:${g.key}`;
                const isOpen = open[id] ?? (searching ? true : i === 0);
                return (
                  <Subfolder
                    key={id}
                    name={g.key}
                    count={g.items.length}
                    lead={
                      <span className="flex text-ink-2">
                        <Icon name="folder" size={18} />
                      </span>
                    }
                    open={isOpen}
                    onToggle={() => onToggle(id, !isOpen)}
                  >
                    {g.items.map((d) => (
                      <DocRow key={d.id} d={d} asking={asking === d.id} onAsk={onAsk} onRemove={onRemove} onOpen={onOpen} busy={busy} />
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
                  {inside.passages.map((p) => {
                    const d = items.find((x) => x.id === p.document_id);
                    return <PassageRow key={p.chunk} p={p} name={d?.name ?? p.document} ext={d?.ext ?? "txt"} />;
                  })}
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
function PassageRow({ p, name, ext }: { p: Passage; name: string; ext: string }) {
  return (
    <li className="list-row !items-start">
      <FileTile ext={ext} />
      <div className="min-w-0 flex-1">
        <b className="block truncate font-bold">{name}</b>
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
  onOpen,
  busy,
}: {
  d: DocItem;
  asking: boolean;
  onAsk: (id: string | null) => void;
  onRemove: (id: string) => void;
  onOpen: (d: DocItem) => void;
  busy: boolean;
}) {
  return (
    <ListRow
      lead={<FileTile ext={d.ext} />}
      title={d.name}
      detail={d.detail}
      state={<Tag tone="green" variant="soft" className="max-sm:hidden">{t.read}</Tag>}
      actions={
        asking ? (
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
            {d.record && <IconButton icon="pencil" label={t.openDoc(d.name)} variant="plain" size="sm" onClick={() => onOpen(d)} />}
            <IconButton icon="trash" label={t.removeDoc(d.name)} variant="plain" size="sm" onClick={() => onAsk(d.id)} />
          </span>
        )
      }
      className={asking ? "!bg-inset" : ""}
    />
  );
}
