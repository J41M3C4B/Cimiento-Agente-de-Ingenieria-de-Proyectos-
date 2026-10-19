import { useState } from "react";
import { es } from "../i18n/es-MX";
import { Icon } from "./icons";
import type { IconName } from "./icons";
import { IconButton } from "./ui";
import type { Page } from "./Shell";

/**
 * The gear of the top bar: what is not a section of the work but how the program is set up and where to ask for
 * help (ADR-028 says who sees what). Each entry takes the person to its page.
 */
export function SettingsMenu({ items, current, onNavigate }: { items: { page: Page; label: string; icon: IconName }[]; current: Page; onNavigate: (page: Page) => void }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="relative">
      <IconButton variant="plain" icon="sliders" label={es.nav.settings} aria-haspopup="menu" aria-expanded={open} onClick={() => setOpen((v) => !v)} />
      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} aria-hidden="true" />
          <div role="menu" aria-label={es.nav.settings} className="absolute right-0 top-14 z-50 w-64 rounded-inset bg-card p-2 shadow-float">
            {items.map(({ page, label, icon }) => (
              <button
                key={page}
                type="button"
                role="menuitem"
                aria-current={current === page ? "page" : undefined}
                onClick={() => {
                  setOpen(false);
                  onNavigate(page);
                }}
                className="flex h-ctl w-full items-center gap-3 rounded-field px-3 text-left text-ui font-bold transition-colors hover:bg-inset aria-[current=page]:bg-inset"
              >
                <Icon name={icon} size={16} className="text-ink-2" />
                {label}
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
