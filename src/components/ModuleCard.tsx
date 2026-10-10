import { MODULE_META } from "./modules";
import type { ModuleId } from "./modules";
import { Metric } from "./ui";

/**
 * A figure of a module (Mi institución and Inicio): the system's `Metric` (its line icon on ink, the label, the figure
 * and one line; a bar takes the accent of the screen) that opens the module when it is pressed.
 */
export function ModuleCard({
  module,
  title,
  figure,
  label,
  approx,
  note,
  fill,
  onOpen,
}: {
  module: ModuleId;
  title: string;
  figure: string;
  label?: string;
  /** the words of the tag that says the figure is an estimate the person gave */
  approx?: string;
  /** a line under the figure that explains it */
  note?: string;
  /** how full something is, 0–100, as a thin bar */
  fill?: number;
  onOpen: () => void;
}) {
  const { icon } = MODULE_META[module];
  return (
    <button type="button" onClick={onOpen} className="block h-full w-full min-w-0 rounded-inset text-left transition-transform hover:-translate-y-0.5">
      <Metric icon={icon} tone="ink" barTone="ac" label={title} value={figure} sub={label} approx={approx} hint={note} fill={fill} />
    </button>
  );
}
