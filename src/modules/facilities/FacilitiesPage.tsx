import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { ModulePage, useNotice } from "../../components/ModulePage";
import { es } from "../../i18n/es-MX";
import { facilitiesOverview } from "./api";
import { FACILITIES_KEY, FacilitiesTab } from "./FacilitiesTab";
import type { FacilitiesView } from "./FacilitiesTab";

/** Instalaciones (ADR-030): the building, its spaces and its equipment, and the state of each one. */
export function FacilitiesPage() {
  const [notice, notify] = useNotice();
  const [view, setView] = useState<FacilitiesView>("spaces");
  // the same query the tab reads: the counts of the tabs
  const overview = useQuery({ queryKey: FACILITIES_KEY, queryFn: facilitiesOverview });
  const data = overview.data;
  const tabs = es.facilities.tabs;
  return (
    <ModulePage
      module="facilities"
      title={es.modules.facilities.title}
      intro={es.modules.facilities.intro}
      notice={notice}
      views={{
        value: view,
        onChange: setView,
        items: [
          { id: "spaces", label: tabs.spaces, count: data?.indicators.spaces },
          { id: "building", label: tabs.building },
          { id: "equipment", label: tabs.equipment, count: data?.indicators.equipment },
          { id: "board", label: tabs.board, count: data?.board.insights.length },
        ],
      }}
    >
      <FacilitiesTab view={view} onView={setView} onNotice={notify} />
    </ModulePage>
  );
}
