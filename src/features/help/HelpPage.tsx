import { Section } from "../../components/ui";
import { es } from "../../i18n/es-MX";

const t = es.help;

/** Plain help: how a project goes, what to do when something fails, and what happens with the data. */
export function HelpPage() {
  return (
    <div className="space-y-6">
      <header className="space-y-1.5">
        <h1 className="text-[24px] font-semibold leading-tight tracking-tight">{t.title}</h1>
        <p className="text-stone-700">{t.intro}</p>
      </header>

      <Section title={t.stepsTitle}>
        <ol className="space-y-3">
          {t.steps.map(([name, text], i) => (
            <li key={name} className="flex gap-4">
              <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-blue-800 font-semibold text-white" aria-hidden="true">
                {i + 1}
              </span>
              <div>
                <p className="font-semibold">{name}</p>
                <p className="text-stone-700">{text}</p>
              </div>
            </li>
          ))}
        </ol>
      </Section>

      <Section title={t.problemsTitle}>
        <dl className="space-y-3">
          {t.problems.map(([q, a]) => (
            <div key={q}>
              <dt className="font-semibold">{q}</dt>
              <dd>{a}</dd>
            </div>
          ))}
        </dl>
      </Section>

      <Section title={t.dataTitle}>
        <ul className="list-disc space-y-1 pl-6">
          {t.data.map((d) => (
            <li key={d}>{d}</li>
          ))}
        </ul>
      </Section>
    </div>
  );
}
