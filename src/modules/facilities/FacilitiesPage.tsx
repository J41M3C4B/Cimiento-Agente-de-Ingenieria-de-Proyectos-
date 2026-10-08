import { Card } from "../../components/ui";
import { ModulePage, useNotice } from "../../components/ModulePage";
import { es } from "../../i18n/es-MX";
import { FacilitiesTab } from "./FacilitiesTab";

/** Instalaciones (ADR-030): the building, its spaces and its equipment, and the state of each one. */
export function FacilitiesPage() {
  const [notice, notify] = useNotice();
  return (
    <ModulePage title={es.modules.facilities.title} intro={es.modules.facilities.intro} notice={notice}>
      <Card>
        <FacilitiesTab onNotice={notify} />
      </Card>
    </ModulePage>
  );
}
