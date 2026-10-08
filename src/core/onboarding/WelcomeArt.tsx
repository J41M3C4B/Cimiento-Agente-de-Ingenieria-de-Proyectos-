import type { CSSProperties } from "react";
import { Icon } from "../../components/icons";
import type { IconName } from "../../components/icons";
import { es } from "../../i18n/es-MX";

const a = es.onboarding.welcome.art;
const NODE_ICONS: IconName[] = ["banknote", "users", "heart", "folder", "file"];
// each bubble a little different in size, so the graph does not look like a row of buttons
const SIZES = [76, 64, 88, 70, 80];
const COLUMNS = a.graph.nodes.length;

/** Step 1: «Mi institución» at the head and, from it, a branch for each part of the program. */
export function OrgGraph() {
  return (
    <div className="onb-graph">
      <span className="onb-head onb-pop">
        <span className="onb-head-icon">
          <Icon name="building" size={24} strokeWidth={2.2} />
        </span>
        {a.graph.head}
      </span>
      <svg className="onb-graph-lines onb-fade" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true" focusable="false">
        {a.graph.nodes.map(([name], i) => {
          const x = ((i + 0.5) / COLUMNS) * 100;
          return <path key={name} d={`M50 0 C50 62 ${x} 38 ${x} 100`} />;
        })}
      </svg>
      <ul className="onb-kids">
        {a.graph.nodes.map(([name, note], i) => (
          <li key={name} className="onb-kid onb-pop" style={{ animationDelay: `${0.15 + i * 0.09}s` }}>
            <span className="onb-kid-bubble" style={{ "--d": `${SIZES[i] ?? 60}px` } as CSSProperties}>
              <Icon name={NODE_ICONS[i] ?? "check"} size={28} strokeWidth={2} />
            </span>
            <b>{name}</b>
            <span>{note}</span>
          </li>
        ))}
      </ul>
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
        <div className="space-y-3" aria-hidden="true">
          {[78, 62, 70, 54].map((w, i) => (
            <div key={i} className="onb-row">
              <span className="onb-ini">
                <Icon name="user" size={15} strokeWidth={2.2} />
              </span>
              <span className="onb-redact" style={{ maxWidth: `${w}%` }} />
            </div>
          ))}
        </div>
        <p>{p.hereNote}</p>
      </div>
      <span className="onb-gate onb-pop" style={{ animationDelay: "0.2s" }} aria-hidden="true">
        <Icon name="shield" size={22} strokeWidth={2.2} />
      </span>
      <div className="onb-box onb-box--sees onb-pop" style={{ animationDelay: "0.35s" }}>
        <h3>
          <Icon name="sparkles" size={18} strokeWidth={2.2} />
          {p.sees}
        </h3>
        <div>
          <p>{p.example}</p>
          <p className="onb-count">
            {p.count}
            <small>{p.people}</small>
          </p>
        </div>
        <p>{p.seesNote}</p>
      </div>
    </div>
  );
}

/** Step 3: a question, its answer and the two things that make it safe to ask. */
export function HelpChat() {
  const h = a.help;
  return (
    <div className="onb-chat">
      <p className="onb-msg onb-msg--ask onb-pop">{h.ask}</p>
      <div className="onb-msg onb-msg--answer onb-pop" style={{ animationDelay: "0.25s" }}>
        <span className="onb-msg-icon">
          <Icon name="help" size={17} strokeWidth={2.2} />
        </span>
        <p>{h.answer}</p>
      </div>
      <div className="onb-pills onb-pop" style={{ animationDelay: "0.5s" }}>
        <span className="onb-pill">
          <Icon name="check" size={16} strokeWidth={2.6} />
          {h.saved}
        </span>
        <span className="onb-pill">
          <Icon name="phone" size={16} strokeWidth={2.2} />
          {h.call}
        </span>
      </div>
    </div>
  );
}
