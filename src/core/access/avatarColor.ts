import { useState } from "react";
import type { Tone } from "../../components/ui";

/** The colors a person may choose for their round mark (red stays for what is wrong, amber is kept: it is one of the series). */
export const AVATAR_TONES: Tone[] = ["violet", "sky", "teal", "cyan", "green", "amber", "rose"];
export const DEFAULT_AVATAR_TONE: Tone = "violet";

const key = (username: string) => `cimiento.avatar.${username}`;

const read = (username: string): Tone => {
  try {
    const v = window.localStorage.getItem(key(username));
    return AVATAR_TONES.find((t) => t === v) ?? DEFAULT_AVATAR_TONE;
  } catch {
    return DEFAULT_AVATAR_TONE;
  }
};

/**
 * The color of a person's avatar: the one they chose, kept for their account, and the same on every page. Whoever has
 * not chosen has violet. Without storage the choice only lasts while the window is open.
 */
export function useAvatarTone(username: string): [Tone, (tone: Tone) => void] {
  const [chosen, setChosen] = useState<{ username: string; tone: Tone }>(() => ({ username, tone: read(username) }));
  // another account came in: read its own color
  const tone = chosen.username === username ? chosen.tone : read(username);
  const choose = (next: Tone) => {
    setChosen({ username, tone: next });
    try {
      window.localStorage.setItem(key(username), next);
    } catch {
      /* the choice stays for this window only */
    }
  };
  return [tone, choose];
}
