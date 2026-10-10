import { Icon } from "../../components/icons";
import { AddSlot, Avatar, Button, FileTile, Folder, IconButton, ListRow, Search, Segmented, Subfolder, Tag, seriesTone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { groupItems, targetOf } from "./documentsModel";
import type { DocCol, DocItem, GroupMode, UploadTarget } from "./documentsModel";

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
            <p className="px-2 py-4 text-ui text-ink-3">{t.noMatch}</p>
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
                      <DocRow key={d.id} d={d} asking={asking === d.id} onAsk={onAsk} onRemove={onRemove} busy={busy} />
                    ))}
                  </Subfolder>
                );
              })}
            </div>
          )}
        </>
      )}
    </Folder>
  );
}

function DocRow({ d, asking, onAsk, onRemove, busy }: { d: DocItem; asking: boolean; onAsk: (id: string | null) => void; onRemove: (id: string) => void; busy: boolean }) {
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
          <IconButton icon="trash" label={t.removeDoc(d.name)} variant="plain" size="sm" onClick={() => onAsk(d.id)} />
        )
      }
      className={asking ? "!bg-inset" : ""}
    />
  );
}
