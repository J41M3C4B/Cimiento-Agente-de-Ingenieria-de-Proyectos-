import { useMutation } from "@tanstack/react-query";
import { Icon } from "../../components/icons";
import { Alert, Button } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { guideExport, toAppError } from "../../lib/tauri";
import type { ConversationView } from "../../lib/types";
import { Said, Thinking } from "./ChatParts";
import { ConversationChat } from "./ConversationChat";
import { Workspace } from "./Workspace";

const t = es.ready;

/** What the guide brings, in the panel. */
function GuideContents() {
  return (
    <div className="flex h-full min-h-0 flex-col">
      <header className="space-y-1 px-6 pb-4 pt-6">
        <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{t.tab}</p>
        <h2 className="text-[15px] font-semibold leading-snug">{t.includesTitle}</h2>
      </header>
      <ul className="min-h-0 flex-1 space-y-2 overflow-y-auto px-6 pb-6">
        {t.includes.map((x) => (
          <li key={x} className="flex gap-2.5 rounded-lg bg-stone-50 px-4 py-3">
            <span aria-hidden className="mt-2.5 h-1.5 w-1.5 shrink-0 rounded-full bg-blue-700" />
            <span>{x}</span>
          </li>
        ))}
      </ul>
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
      {make.isError && <Alert tone="warn">{toAppError(make.error).message}</Alert>}
      {make.data && (
        <Said>
          {t.done(make.data.file_name)}
          <span className="mt-1 block break-all text-[13px] text-stone-600">
            {t.where}: {make.data.path}
          </span>
        </Said>
      )}
    </div>
  );

  const next = (
    <div className="space-y-3">
      <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{t.nextStep}</p>
      <Button variant="primary" className="w-full" onClick={() => make.mutate()} disabled={working}>
        <Icon name="download" size={17} />
        {make.isPending ? t.generating : make.data ? t.again : t.generate}
      </Button>
      <Button variant="ghost" size="sm" className="w-full" onClick={onBack} disabled={working}>
        <Icon name="back" size={15} />
        {es.review.back}
      </Button>
      <p className="text-[12.5px] text-stone-600">{t.changeNote}</p>
    </div>
  );

  return (
    <Workspace panelOpen={panelOpen} readingId={view.project.call_reading_id} project={<GuideContents />} projectLabel={t.tab} footer={next}>
      <ConversationChat view={view} onView={onView} disabled={working} tail={tail} later={{ placeholder: t.placeholder, hint: t.hint }} />
    </Workspace>
  );
}
