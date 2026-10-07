import type { ChangeEvent, DragEvent, ReactNode } from "react";
import { useState } from "react";
import { Icon } from "../icons";
import type { IconName } from "../icons";
import { Inset } from "./Surface";
import { Tile } from "./Tag";
import type { Tone } from "./Tag";

/** The small label over a block («PRÓXIMOS PASOS»). */
export function Eyebrow({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <p className={`eyebrow ${className}`}>{children}</p>;
}

const FILE_TONE: Record<string, Tone> = { pdf: "red", doc: "sky", docx: "sky", xls: "green", xlsx: "green", csv: "green", ppt: "amber", pptx: "amber" };

/** The square of a file, in the color of its kind: PDF red, Word blue, Excel and CSV green, PowerPoint yellow, else gray. */
export function FileTile({ ext, small }: { ext: string; small?: boolean }) {
  const e = ext.toLowerCase().replace(".", "");
  return <span className={`file-tile tone-${FILE_TONE[e] ?? "neutral"} ${small ? "file-tile--sm" : ""}`}>{e.slice(0, 4).toUpperCase()}</span>;
}

/** One line of a list: a leading mark, a title with its detail, a state and (on hover) actions. */
export function ListRow({ lead, title, detail, state, actions, className = "" }: { lead?: ReactNode; title: ReactNode; detail?: ReactNode; state?: ReactNode; actions?: ReactNode; className?: string }) {
  return (
    <li className={`list-row group ${className}`}>
      {lead}
      <div className="min-w-0 flex-1">
        <b className="block truncate font-bold">{title}</b>
        {detail && <small className="block truncate text-small font-medium text-ink-3">{detail}</small>}
      </div>
      {state}
      {actions && <div className="list-row-actions">{actions}</div>}
    </li>
  );
}

/**
 * A group inside a tray: its own small tab, a border around what it holds, hanging from the trunk line of its parent
 * (put them inside `<div className="subfolders">`). It opens and closes; `onUpload` adds the «Subir a «…»» button.
 */
export function Subfolder({
  name,
  count,
  lead,
  open,
  onToggle,
  uploadLabel,
  dropHint,
  onUpload,
  onDropFile,
  children,
}: {
  name: string;
  count: number;
  /** an avatar or a folder icon */
  lead?: ReactNode;
  open: boolean;
  onToggle: () => void;
  uploadLabel?: string;
  dropHint?: string;
  onUpload?: () => void;
  onDropFile?: (file: File) => void;
  children: ReactNode;
}) {
  const [over, setOver] = useState(false);
  const drag = (e: DragEvent) => {
    if (!onDropFile) return;
    e.preventDefault();
    setOver(true);
  };
  return (
    <section
      className={`subfolder ${over ? "subfolder--over" : ""}`}
      data-open={open}
      onDragOver={drag}
      onDragLeave={(e) => !e.currentTarget.contains(e.relatedTarget as Node | null) && setOver(false)}
      onDrop={(e) => {
        if (!onDropFile) return;
        e.preventDefault();
        setOver(false);
        const file = e.dataTransfer.files[0];
        if (file) onDropFile(file);
      }}
    >
      <button type="button" className="subfolder-tab" aria-expanded={open} onClick={onToggle}>
        <span className="subfolder-chev" aria-hidden="true">
          <Icon name="down" size={16} />
        </span>
        {lead}
        <b className="truncate">{name}</b>
        <span className="subfolder-count tabular">{count}</span>
      </button>
      {open && (
        <div className="subfolder-box">
          <ul>{children}</ul>
          {onUpload && uploadLabel && (
            <div className="flex flex-wrap items-center gap-3 px-1 pb-0.5 pt-1.5">
              <button type="button" onClick={onUpload} className="add-slot !min-h-ctl max-w-full !px-4 !py-2 text-left text-small">
                <Icon name="upload" size={16} />
                {uploadLabel}
              </button>
              {dropHint && <span className="text-small text-ink-3 max-sm:hidden">{dropHint}</span>}
            </div>
          )}
        </div>
      )}
    </section>
  );
}

/** Where a file is dropped or chosen; once chosen it becomes a row with the file's name. */
export function DropZone({
  label,
  hint,
  file,
  onFile,
  accept,
  changeHint,
  error,
  multiple,
  ariaLabel,
}: {
  label: string;
  hint?: string;
  file?: { name: string; size: string; ext: string } | null;
  onFile: (file: File) => void;
  accept?: string;
  changeHint?: string;
  error?: string;
  multiple?: boolean;
  ariaLabel?: string;
}) {
  const [over, setOver] = useState(false);
  const pick = (e: ChangeEvent<HTMLInputElement>) => {
    const f = e.target.files?.[0];
    if (f) onFile(f);
  };
  return (
    <div>
      <label
        className={`dropzone ${file ? "dropzone--filled" : ""} ${over ? "dropzone--over" : ""} ${error ? "field--invalid" : ""}`}
        onDragOver={(e) => {
          e.preventDefault();
          setOver(true);
        }}
        onDragLeave={() => setOver(false)}
        onDrop={(e) => {
          e.preventDefault();
          setOver(false);
          const f = e.dataTransfer.files[0];
          if (f) onFile(f);
        }}
      >
        <input type="file" accept={accept} multiple={multiple} aria-label={ariaLabel} onChange={pick} />
        {file ? (
          <>
            <FileTile ext={file.ext} />
            <span className="min-w-0 flex-1">
              <b className="block truncate font-bold">{file.name}</b>
              <small className="text-small text-ink-3">{file.size}{changeHint ? ` · ${changeHint}` : ""}</small>
            </span>
          </>
        ) : (
          <>
            <span className="grid h-ctl w-ctl place-items-center rounded-pill bg-inset text-ink-2">
              <Icon name="upload" size={20} />
            </span>
            <span>
              <b className="font-bold">{label}</b>
            </span>
            {hint && <span className="text-small text-ink-3">{hint}</span>}
          </>
        )}
      </label>
      {error && (
        <span role="alert" className="field-error">
          <Icon name="alert" size={15} />
          {error}
        </span>
      )}
    </div>
  );
}

/** A message of the chat: the assistant's on the left (soft), the person's on the right (black). */
export function Bubble({ from, avatar, children }: { from: "assistant" | "person"; avatar?: ReactNode; children: ReactNode }) {
  const me = from === "person";
  return (
    <div className={`anim-rise flex max-w-[92%] items-start gap-2.5 ${me ? "flex-row-reverse self-end" : ""}`}>
      {avatar}
      <div className={`bubble text-body ${me ? "bubble--me" : ""}`}>{children}</div>
    </div>
  );
}

/** The box where the person writes, with its round send button. */
export function Composer({ children, send }: { children: ReactNode; send: ReactNode }) {
  return (
    <div className="composer">
      {children}
      {send}
    </div>
  );
}

/** A date as a big number: the weekday above, the month and the days left below. */
export function Calendar({ weekday, day, month, note, icon = "calendar" }: { weekday: string; day: string; month: string; note?: string; icon?: IconName }) {
  return (
    <div className="calendar">
      <div className="flex items-center justify-center gap-1.5 text-ui font-bold text-ink-2">
        <Icon name={icon} size={16} />
        {weekday}
      </div>
      <div className="tabular my-2 text-display font-normal tracking-tighter">{day}</div>
      <div className="text-small font-semibold text-ink-2">{month}</div>
      {note && <div className="mt-1 text-small font-semibold text-ink-2">{note}</div>}
    </div>
  );
}

/**
 * «Lo siguiente»: the one thing to do now, in the color of the project (use it inside a `Folder`). The whole box can
 * be pressed; its round button opens into a pill with the word on hover or focus. It is the only «continue».
 */
export function NextBox({ title, detail, goLabel, onGo, eyebrow }: { title: string; detail?: string; goLabel: string; onGo: () => void; eyebrow?: string }) {
  return (
    <button type="button" className="next-box" onClick={onGo}>
      {eyebrow && <Eyebrow>{eyebrow}</Eyebrow>}
      <span className="text-subtitle font-bold tracking-tight">{title}</span>
      {detail && <span className="max-w-[46ch] text-body text-ink-2">{detail}</span>}
      <span className="next-go" aria-hidden="true">
        <span>{goLabel}</span>
        <Icon name="next" size={20} />
      </span>
    </button>
  );
}

/** One figure on an inset: its square with an icon, its label, the number, a detail and (optionally) a bar. */
export function Metric({ icon, tone, label, value, sub, fill, note }: { icon: IconName; tone: Tone; label: string; value: string; sub?: string; fill?: number; note?: string }) {
  return (
    <Inset className="flex flex-col gap-3">
      <div className="flex items-center gap-2.5">
        <Tile icon={icon} tone={tone} small />
        <span className="text-ui font-semibold text-ink-2">{label}</span>
      </div>
      <div>
        <div className="tabular whitespace-nowrap text-hero font-normal tracking-tight">{value}</div>
        {sub && <div className="mt-0.5 text-small text-ink-3">{sub}</div>}
      </div>
      {note && (
        <p className="flex items-start gap-1.5 text-small font-semibold text-amber-ink">
          <Icon name="warn" size={14} className="mt-0.5" />
          {note}
        </p>
      )}
      {fill !== undefined && (
        <div className={`bar tone-${tone} mt-auto !h-1.5`}>
          <i style={{ width: `${Math.max(0, Math.min(100, fill))}%` }} />
        </div>
      )}
    </Inset>
  );
}

/** A person on an inset: avatar, name, a detail and a state on the right. */
export function PersonCard({ avatar, name, detail, state }: { avatar: ReactNode; name: string; detail?: string; state?: ReactNode }) {
  return (
    <Inset className="flex items-center gap-3 !py-3">
      {avatar}
      <div className="min-w-0 flex-1">
        <b className="block truncate font-bold leading-tight">{name}</b>
        {detail && <span className="block truncate text-small text-ink-3">{detail}</span>}
      </div>
      {state}
    </Inset>
  );
}

/** The dashed slot that adds something («+ Agregar persona»); it takes the place of loose «Agregar» buttons. */
export function AddSlot({ children, onClick, disabled }: { children: ReactNode; onClick: () => void; disabled?: boolean }) {
  return (
    <button type="button" onClick={onClick} disabled={disabled} className="add-slot w-full">
      <Icon name="plus" size={18} />
      {children}
    </button>
  );
}


