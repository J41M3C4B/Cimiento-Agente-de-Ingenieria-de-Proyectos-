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
