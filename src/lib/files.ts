import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

/** The bytes of a file in base64, as the commands that take a file receive it. */
export function toBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const s = String(reader.result);
      resolve(s.slice(s.indexOf(",") + 1));
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

/** `key`: what is being read, when its reader named it (a document of the Expediente read in the background). */
export type ReadingProgress = { key?: string | null; done: number; total: number };

/**
 * How a reading of scanned pages or photos goes (OCR, ADR-035) while `active`: Rust says how many pages are read.
 * `null` until it says something, or when no picture is being read.
 */
export function useReadingProgress(active: boolean): ReadingProgress | null {
  const [progress, setProgress] = useState<ReadingProgress | null>(null);
  useEffect(() => {
    if (!active) {
      setProgress(null);
      return;
    }
    let stop: (() => void) | undefined;
    let gone = false;
    listen<ReadingProgress>("reading-progress", (e) => setProgress(e.payload))
      .then((unlisten) => (gone ? unlisten() : (stop = unlisten)))
      // outside the program (the tests) there is nobody to hear
      .catch(() => {});
    return () => {
      gone = true;
      stop?.();
      setProgress(null);
    };
  }, [active]);
  return progress;
}
