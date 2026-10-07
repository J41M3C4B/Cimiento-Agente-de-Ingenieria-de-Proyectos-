// @vitest-environment node
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

/**
 * The guard of the visual system (docs/13 §14). Screens may only use the names defined in src/index.css and the
 * components of src/components/ui/. This test reads every screen and fails with the file, the line and the rule,
 * so a loose color, a one-off size or a leftover Tailwind scale cannot get in unnoticed.
 */
const SRC = fileURLToPath(new URL(".", import.meta.url));

const walk = (dir: string): string[] =>
  readdirSync(dir).flatMap((name: string) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });

// the catalog and the tokens are where the values live; tests and texts are not interface code
const screens = walk(SRC).filter((f) => /\.(tsx?)$/.test(f) && !/\.test\.tsx?$/.test(f) && !f.includes(`${join("components", "ui")}`) && !f.includes("i18n") && !f.includes("design.test"));

const COLORS = "ink|ink-2|ink-3|on-ink|onc|sky-ink|red-ink|amber-ink|green-ink|sky|violet|rose|red|amber|green|teal|cyan|card|dock|inset|line|canvas|frame|current|transparent|inherit";
const SIZES = "caption|small|ui|body|heading|subtitle|title|hero|display";
const ALIGN = "left|center|right|justify|start|end|wrap|nowrap|balance|pretty|ellipsis|clip";
const RADII = "tick|field|inset|card|frame|pill|none";

type Rule = { name: string; test: RegExp; why: string };
const rules: Rule[] = [
  { name: "hex color", test: /["'`(\s]#[0-9a-fA-F]{6,8}\b|["'`]#[0-9a-fA-F]{3}["'`]/, why: "colors are tokens (bg-card, text-ink-2, bg-sky…) defined in src/index.css" },
  { name: "Tailwind palette scale", test: /\b(?:bg|text|border|ring|from|to|via|fill|stroke|divide|shadow|outline|accent|decoration|placeholder)-(?:stone|slate|gray|zinc|neutral|blue|navy|brass|indigo|purple|pink|orange|yellow|lime|emerald|fuchsia|rose|sky|cyan|teal|green|red|amber|violet)-\d{2,3}\b/, why: "the numbered Tailwind colors were removed; use the named tokens" },
  { name: "white or black", test: /\b(?:bg|text|border|ring|fill|stroke|divide)-(?:white|black)\b/, why: "use bg-card / text-on-ink / bg-ink" },
  { name: "arbitrary value for identity", test: /\b(?:text|rounded(?:-[a-z]{1,2})?|shadow|bg|border(?:-[a-z])?|ring|font|leading|tracking|p[xytrbl]?|m[xytrbl]?|gap(?:-[xy])?|space-[xy]|size|inset|top|bottom|left|right)-\[/, why: "use a token (text-ui, rounded-field, shadow-card…); only layout sizes (w-, h-, min-, max-, grid-cols-) may be arbitrary" },
  { name: "Tailwind default size", test: /\btext-(?:xs|sm|base|lg|xl|[2-9]xl)\b/, why: "text sizes are caption, small, ui, body, heading, subtitle, title, hero, display" },
  { name: "Tailwind default radius", test: /\brounded(?:-(?:t|b|l|r|tl|tr|bl|br|s|e))?(?:-(?:xs|sm|md|lg|xl|2xl|3xl|4xl|full))?(?=["'`\s}]|$)(?<!rounded-(?:t|b|l|r|tl|tr|bl|br|s|e))/, why: "radii are rounded-tick, field, inset, card, frame, pill" },
  { name: "Tailwind default shadow", test: /\bshadow(?:-(?:2xs|xs|sm|md|lg|xl|2xl|inner|lift|panel))?(?=["'`\s}]|$)/, why: "shadows are shadow-card, shadow-frame, shadow-float" },
  { name: "unknown text-*", test: new RegExp(`\\btext-(?!(?:${SIZES}|${ALIGN}|${COLORS})(?![\\w-]))[a-z][\\w-]*`), why: "unknown text token (sizes or colors from src/index.css)" },
  { name: "unknown bg-*", test: new RegExp(`\\bbg-(?!(?:${COLORS}|none|gradient|linear|radial|conic|clip|fixed|local|scroll|center|cover|contain|repeat|no-repeat)(?![\\w-]))[a-z][\\w-]*`), why: "unknown background token" },
  { name: "unknown rounded-*", test: new RegExp(`\\brounded-(?:(?:t|b|l|r|tl|tr|bl|br|s|e)-)?(?!(?:${RADII})(?![\\w-]))[a-z0-9][\\w-]*`), why: "radii are rounded-tick, field, inset, card, frame, pill" },
  { name: "colored style", test: /style=\{\{[^}]*\b(?:color|background|backgroundColor|borderColor|borderRadius|fontSize|fontWeight|boxShadow)\s*:/, why: "colors, radii, sizes and shadows are tokens; style is only for measures that come from data (widths, percentages)" },
];

describe("el sistema visual (docs/13 §14)", () => {
  it("encuentra las pantallas", () => {
    expect(screens.length).toBeGreaterThan(20);
  });

  for (const rule of rules) {
    it(`ninguna pantalla usa: ${rule.name}`, () => {
      const found: string[] = [];
      for (const file of screens) {
        readFileSync(file, "utf8")
          .split("\n")
          .forEach((line: string, i: number) => {
            const code = line.trim();
            if (code.startsWith("*") || code.startsWith("//") || code.startsWith("/*")) return;
            if (rule.test.test(line)) found.push(`${relative(SRC, file)}:${i + 1}  ${line.trim().slice(0, 120)}`);
          });
      }
      expect(found, `${rule.why}\n${found.slice(0, 40).join("\n")}`).toEqual([]);
    });
  }

  it("solo hay un archivo de estilos, y es el de los tokens", () => {
    expect(walk(SRC).filter((f) => f.endsWith(".css")).map((f) => relative(SRC, f))).toEqual(["index.css"]);
  });
});
