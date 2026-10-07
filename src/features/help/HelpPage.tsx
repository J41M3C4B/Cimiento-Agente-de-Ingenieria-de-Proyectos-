import { Avatar, Card, Disclosure, PageHeader } from "../../components/ui";
import { es } from "../../i18n/es-MX";

const t = es.help;

/** Plain help: how a project goes, what to do when something fails, and what happens with the data. */
export function HelpPage() {
  return (
    <div className="space-y-6">
      <PageHeader title={t.title} intro={t.intro} />

      <div className="grid items-start gap-4 min-[1000px]:grid-cols-2">
        <Card as="section">
          <h2 className="text-heading font-bold">{t.stepsTitle}</h2>
          <ol className="mt-4 space-y-3">
            {t.steps.map(([name, text], i) => (
              <li key={name} className="flex gap-4">
                <Avatar name={String(i + 1)} tone="ink" />
                <div className="min-w-0">
                  <p className="text-ui font-bold">{name}</p>
                  <p className="text-ui text-ink-2">{text}</p>
                </div>
              </li>
            ))}
          </ol>
        </Card>

        <div className="space-y-4">
          <Card as="section">
            <h2 className="text-heading font-bold">{t.problemsTitle}</h2>
            <div className="mt-4 space-y-3">
              {t.problems.map(([q, a], i) => (
                <Disclosure key={q} title={q} defaultOpen={i === 0}>
                  <p className="text-ui text-ink-2">{a}</p>
                </Disclosure>
              ))}
            </div>
          </Card>

          <Card as="section">
            <h2 className="text-heading font-bold">{t.dataTitle}</h2>
            <ul className="mt-4 list-disc space-y-2 pl-6 text-ui text-ink-2">
              {t.data.map((d) => (
                <li key={d}>{d}</li>
              ))}
            </ul>
          </Card>
        </div>
      </div>
    </div>
  );
}
