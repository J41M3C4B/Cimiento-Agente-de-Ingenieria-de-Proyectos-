import { useEffect, useState } from "react";

export type Theme = "light" | "dark";
const KEY = "cimiento.theme";

const stored = (): Theme | null => {
  try {
    const v = window.localStorage.getItem(KEY);
    return v === "light" || v === "dark" ? v : null;
  } catch {
    return null;
  }
};

const system = (): Theme => (typeof window.matchMedia === "function" && window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");

/** The theme of the window: the one the person chose, or else the one of the system. Choosing it is remembered. */
export function useTheme(): [Theme, () => void] {
  const [theme, setTheme] = useState<Theme>(() => stored() ?? system());
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
  const toggle = () => {
    const next: Theme = theme === "dark" ? "light" : "dark";
    setTheme(next);
    try {
      window.localStorage.setItem(KEY, next);
    } catch {
      /* without storage the choice only lasts while the window is open */
    }
  };
  return [theme, toggle];
}
