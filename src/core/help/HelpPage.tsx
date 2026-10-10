import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { ManualText } from "../../components/ManualHelp";
import { Avatar, Card, Disclosure, PageHeader, Search } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { manualEntry, manualSearch } from "../../lib/tauri";

const t = es.help;
const m = es.manual;

/** One result of the search: its title, and the whole entry when it is opened. */
function Hit({ id, title }: { id: string; title: string }) {
  const entry = useQuery({ queryKey: ["manual", id], queryFn: () => manualEntry(id), staleTime: Infinity });
  return <Disclosure title={title}>{entry.data ? <ManualText entry={entry.data} /> : null}</Disclosure>;
}

/** The search in the manual (ADR-034): in the program, so it works without internet and without the AI. */
function ManualSearch() {
  const [text, setText] = useState("");
  const [query, setQuery] = useState("");
  useEffect(() => {
    const wait = setTimeout(() => setQuery(text.trim()), 250);
    return () => clearTimeout(wait);
  }, [text]);
  const hits = useQuery({ queryKey: ["manual-search", query], queryFn: () => manualSearch(query), enabled: query.length >= 3 });
  return (
    <Card as="section">
      <h2 className="text-heading font-bold">{m.searchTitle}</h2>
      <p className="mt-1 text-ui text-ink-2">{m.searchHelp}</p>
      <Search label={m.searchLabel} placeholder={m.searchPlaceholder} value={text} onChange={(e) => setText(e.target.value)} className="mt-4" />
      {query.length >= 3 && hits.data && (
        <div className="mt-4 space-y-3">
          {hits.data.length === 0 ? <p className="text-ui text-ink-2">{m.noResults}</p> : hits.data.map((h) => <Hit key={h.id} id={h.id} title={h.title} />)}
        </div>
      )}
    </Card>
  );
}

/** Plain help: how a project goes, what to do when something fails, and what happens with the data. */
export function HelpPage() {
  return (
    <div className="space-y-6">
      <PageHeader title={t.title} intro={t.intro} />

      <ManualSearch />

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
