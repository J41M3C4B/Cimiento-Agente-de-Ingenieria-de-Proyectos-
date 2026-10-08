import { useQueryClient } from "@tanstack/react-query";
import { Card } from "../../components/ui";
import { ModulePage, useNotice } from "../../components/ModulePage";
import { es } from "../../i18n/es-MX";
import type { ProfileView } from "../../lib/types";
import { FINANCE_KEY } from "../finance/api";
import { CareTab } from "./CareTab";

/** Beneficiarios (ADR-029): the people served, the waiting list and the board. */
export function CarePage() {
  const qc = useQueryClient();
  const [notice, notify] = useNotice();
  // the profile keeps the anonymous lines of the people served, and their stay fees are income of the balance
  const onProfile = (p: ProfileView) => {
    qc.setQueryData(["profile"], p);
    void qc.invalidateQueries({ queryKey: FINANCE_KEY });
  };
  return (
    <ModulePage title={es.modules.people.title} intro={es.modules.people.intro} notice={notice}>
      <Card>
        <CareTab onProfile={onProfile} onNotice={notify} />
      </Card>
    </ModulePage>
  );
}
