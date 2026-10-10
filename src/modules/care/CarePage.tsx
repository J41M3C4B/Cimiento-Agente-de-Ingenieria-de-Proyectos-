import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { ModulePage, useNotice } from "../../components/ModulePage";
import { es } from "../../i18n/es-MX";
import type { ProfileView } from "../../lib/types";
import { FINANCE_KEY } from "../finance/api";
import { careOverview } from "./api";
import { CARE_KEY, CareTab } from "./CareTab";
import type { CareView } from "./CareTab";

/** Beneficiarios (ADR-029): the people served, the waiting list and the board. */
export function CarePage() {
  const qc = useQueryClient();
  const [notice, notify] = useNotice();
  const [view, setView] = useState<CareView>("people");
  // the same query the tab reads: the counts of the tabs
  const overview = useQuery({ queryKey: CARE_KEY, queryFn: careOverview });
  const board = overview.data?.board;
  // the profile keeps the anonymous lines of the people served, and their stay fees are income of the balance
  const onProfile = (p: ProfileView) => {
    qc.setQueryData(["profile"], p);
    void qc.invalidateQueries({ queryKey: FINANCE_KEY });
  };
  const tabs = es.care.tabs;
  return (
    <ModulePage
      module="people"
      title={es.modules.people.title}
      intro={es.modules.people.intro}
      notice={notice}
      views={{
        value: view,
        onChange: setView,
        items: [
          { id: "people", label: tabs.people, count: board?.indicators.served },
          { id: "board", label: tabs.board },
          { id: "waitlist", label: tabs.waitlist, count: board?.waiting },
        ],
      }}
    >
      <CareTab view={view} onProfile={onProfile} onNotice={notify} />
    </ModulePage>
  );
}
