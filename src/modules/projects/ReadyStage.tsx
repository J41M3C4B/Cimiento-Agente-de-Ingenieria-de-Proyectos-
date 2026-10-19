import { useMutation } from "@tanstack/react-query";
import { Icon } from "../../components/icons";
import { Alert, Button, Eyebrow, FileTile, Inset, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { guideExport, toAppError } from "../../lib/tauri";
import type { ConversationView } from "../../lib/types";
import { Said, Thinking } from "./ChatParts";
import { ConversationChat } from "./ConversationChat";
import { Workspace } from "./Workspace";

const t = es.ready;

/** What the guide brings, in the panel; once it is made, the file itself on top. */
function GuideContents({ file }: { file: { file_name: string } | undefined }) {
  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto">
        {file && (
          <Inset className="space-y-3">
            <div className="flex items-center justify-between gap-3">
              <FileTile ext="docx" />
              <Tag tone="green" icon="check">
                {t.made}
              </Tag>
            </div>
            <b className="block break-words font-bold">{file.file_name}</b>
          </Inset>
        )}
        <h2 className="text-ui font-bold">{t.includesTitle}</h2>
        <ul className="space-y-2">
          {t.includes.map((x) => (
            <li key={x} className="flex items-start gap-3 text-ui">
              <span className="mt-0.5 grid h-6 w-6 shrink-0 place-items-center rounded-pill bg-green text-onc">
                <Icon name="check" size={13} strokeWidth={3} />
              </span>
              <span>{x}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

/**
 * The last step, as the end of the same chat: the assistant says the project is ready and, when the person asks for
 * the guide from the panel, says where the Word file was left. The panel's footer is the button and the way back.
 */
export function ReadyStage({
  view,
  onView,
  onBack,
  busy: parentBusy,
  panelOpen,
}: {
  view: ConversationView;
  onView: (v: ConversationView) => void;
  onBack: () => void;
  busy: boolean;
  panelOpen: boolean;
}) {
  const make = useMutation({ mutationFn: () => guideExport(view.project.id) });
  const working = make.isPending || parentBusy;

  const tail = (
    <div className="space-y-6">
      <Said>{t.start}</Said>
      {make.isPending && <Thinking phrases={[t.generating]} />}
      {make.isError && <Alert tone="error">{toAppError(make.error).message}</Alert>}
      {make.data && (
        <Said>
          {t.done(make.data.file_name)}
          <span className="mt-1 block break-all text-small text-ink-3">
            {t.where}: {make.data.path}
          </span>
        </Said>
      )}
    </div>
  );

  const next = (
    <div className="space-y-3">
      <Eyebrow>{t.nextStep}</Eyebrow>
      <Button variant="primary" className="w-full" onClick={() => make.mutate()} disabled={working}>
        <Icon name="download" size={18} />
        {make.isPending ? t.generating : make.data ? t.again : t.generate}
      </Button>
      <Button variant="ghost" size="sm" className="w-full" onClick={onBack} disabled={working}>
        <Icon name="back" size={16} />
        {es.review.back}
      </Button>
      <p className="text-small text-ink-3">{t.changeNote}</p>
    </div>
  );

  return (
    <Workspace panelOpen={panelOpen} readingId={view.project.call_reading_id} project={<GuideContents file={make.data} />} projectLabel={t.tab} footer={next}>
      <ConversationChat view={view} onView={onView} disabled={working} tail={tail} later={{ placeholder: t.placeholder, hint: t.hint }} />
    </Workspace>
  );
}
