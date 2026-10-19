import { useEffect, useState } from "react";
import type { CSSProperties } from "react";
import { Icon } from "../../components/icons";
import { Disclosure } from "../../components/ui";
import { es } from "../../i18n/es-MX";

const a = es.onboarding.welcome.art;

/**
 * Step 1: «Mi institución» and, floating around it, the modules with their own isotipo and one line each. They come in
 * one by one, they bob a little, and a spotlight moves from one to the next so the picture is never still. With the
 * system set to reduce motion, everything shows at once and nothing moves.
 */
export function ModuleBubbles() {
  const nodes = a.graph.nodes;
  const [active, setActive] = useState(-1);
  useEffect(() => {
    if (typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let every: ReturnType<typeof setInterval> | undefined;
    // the spotlight starts when the last bubble has come in
    const start = setTimeout(() => {
      setActive(0);
      every = setInterval(() => setActive((i) => (i + 1) % nodes.length), 2600);
    }, 400 + nodes.length * 400 + 600);
    return () => {
      clearTimeout(start);
      if (every) clearInterval(every);
    };
  }, [nodes.length]);

  const half = Math.ceil(nodes.length / 2);
  const row = (items: typeof nodes, from: number) => (
    <ul className="onb-row">
      {items.map(([name, note, id], k) => {
        const i = from + k;
        return (
          <li key={id} data-m={id} data-active={i === active} className="onb-bubble onb-pop" style={{ animationDelay: `${0.4 + i * 0.4}s` }}>
            <span className="onb-float" style={{ "--d": `${(i % 3) * -1.3}s`, "--t": `${4.6 + (i % 3) * 0.7}s` } as CSSProperties}>
              <span className="onb-orb">
                <img src={`/SVG/isotipo_${id}.svg`} alt="" aria-hidden="true" draggable={false} />
              </span>
            </span>
            <b>{name}</b>
            <span className="onb-bubble-note">{note}</span>
          </li>
        );
      })}
    </ul>
  );
  return (
    <div className="onb-cloud">
      {row(nodes.slice(0, half), 0)}
      <span className="onb-head onb-pop">
        <span className="onb-head-icon">
          <Icon name="building" size={20} strokeWidth={2.2} />
        </span>
        {a.graph.head}
      </span>
      {row(nodes.slice(half), half)}
    </div>
  );
}

/** Step 2: the names stay on this computer; the automatic help only gets a count. */
export function PrivacyFlow() {
  const p = a.privacy;
  return (
    <div className="onb-flow">
      <div className="onb-box onb-box--here onb-pop">
        <h3>
          <Icon name="lock" size={18} strokeWidth={2.2} />
          {p.here}
        </h3>
        <ul className="onb-names" aria-hidden="true">
          {p.names.map((name) => (
            <li key={name} className="onb-name">
              <span className="onb-ini">
                <Icon name="user" size={15} strokeWidth={2.2} />
              </span>
              <span>{name}</span>
            </li>
          ))}
        </ul>
        <p>{p.hereNote}</p>
      </div>
      <div className="onb-wire onb-pop" style={{ animationDelay: "0.2s" }} aria-hidden="true">
        <span className="onb-gate">
          <Icon name="shield" size={22} strokeWidth={2.2} />
        </span>
      </div>
      <div className="onb-box onb-box--sees onb-pop" style={{ animationDelay: "0.35s" }}>
        <h3>
          <Icon name="sparkles" size={18} strokeWidth={2.2} />
          {p.sees}
        </h3>
        <p className="onb-count">
          {p.count}
          <small>{p.people}</small>
        </p>
        <p>{p.seesNote}</p>
      </div>
    </div>
  );
}

/** Step 3: the page of «Ayuda» as the person will find it (the real questions, the first one open) and the two things that make it safe to ask. */
export function HelpPreview() {
  const h = a.help;
  const problems = es.help.problems.slice(0, 2);
  return (
    <div className="onb-help">
      <div className="onb-help-card onb-pop">
        <h3>{es.help.problemsTitle}</h3>
        {problems.map(([q, answer], i) => (
          <Disclosure key={q} title={q} defaultOpen={i === 0}>
            <p>{answer}</p>
          </Disclosure>
        ))}
      </div>
      <ul className="onb-help-list onb-pop" style={{ animationDelay: "0.25s" }}>
        <li>
          <Icon name="check" size={18} strokeWidth={2.6} />
          {h.saved}
        </li>
        <li>
          <Icon name="phone" size={18} strokeWidth={2.2} />
          {h.call}
        </li>
      </ul>
    </div>
  );
}
