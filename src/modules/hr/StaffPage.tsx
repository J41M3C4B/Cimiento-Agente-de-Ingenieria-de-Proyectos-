import { useQueryClient } from "@tanstack/react-query";
import { ModulePage, useNotice } from "../../components/ModulePage";
import { es } from "../../i18n/es-MX";
import type { ProfileView } from "../../lib/types";
import { FINANCE_KEY } from "../finance/api";
import { StaffTab } from "./StaffTab";

/** Personal (ADR-027): the records of whoever works in the institution and the catalog of positions. */
export function StaffPage() {
  const qc = useQueryClient();
  const [notice, notify] = useNotice();
  // the profile keeps the anonymous lines of the staff, and the payroll is an expense of the balance
  const onProfile = (p: ProfileView) => {
    qc.setQueryData(["profile"], p);
    void qc.invalidateQueries({ queryKey: FINANCE_KEY });
  };
  return (
    <ModulePage module="staff" title={es.modules.staff.title} intro={es.modules.staff.intro} notice={notice}>
      <StaffTab onProfile={onProfile} onNotice={notify} />
    </ModulePage>
  );
}
