import { useQuery } from "@tanstack/react-query";
import { Icon } from "../../components/icons";
import { Avatar, Bar } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { profileGet } from "../../lib/tauri";
import { accessTeam } from "../access/api";
import { useAvatarTone } from "../access/avatarColor";
import { useSession } from "../access/session";
import { useFillGaps } from "../profile/gaps";

const t = es.home.institutionCard;
const SHOWN = 4;

/**
 * «Mi institución» as a dark card (Inicio): while the data is not complete it says how far they are, with a bar; once
 * complete it only keeps the name and the people who have access. The whole card opens «Mi institución».
 */
export function InstitutionCard({ onOpen }: { onOpen: () => void }) {
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  const team = useQuery({ queryKey: ["access", "team"], queryFn: accessTeam });
  const fill = useFillGaps();
  const access = useSession();
  const [mine] = useAvatarTone(access?.session.username ?? "");

  const name = profile.data?.input.institution.name?.trim() || es.nav.profile;
  const complete = fill.ready && fill.gaps.length === 0;
  // how far the data are, as Rust counts it: it reads 100 % only with nothing missing
  const percent = fill.percent;
  const people = team.data ?? [];

  return (
    <button type="button" onClick={onOpen} className="inst-card">
      <span className="flex items-center justify-between gap-3">
        <b className="inst-name" title={name}>
          {name}
        </b>
        <Icon name="next" size={18} />
      </span>
      {complete ? (
        people.length > 0 && (
          <span className="inst-people" role="group" aria-label={t.people}>
            {people.slice(0, SHOWN).map((p) => (
              <Avatar key={p.display_name} name={p.display_name} size="sm" tone={p.display_name === access?.session.display_name ? mine : undefined} />
            ))}
            {people.length > SHOWN && <span className="inst-more">{t.more(people.length - SHOWN)}</span>}
          </span>
        )
      ) : (
        fill.ready && (
          <span className="flex flex-col gap-1.5">
            <Bar percent={percent} label={t.progress(percent)} />
            <span className="inst-caption">{`${t.progress(percent)} · ${es.profile.completion.pending(fill.gaps.length)}`}</span>
          </span>
        )
      )}
    </button>
  );
}
