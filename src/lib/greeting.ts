export type DayPart = "morning" | "afternoon" | "evening";

/** The part of the day an hour (0–23) belongs to: morning from 5, afternoon from 12, evening from 19 until 5. */
export function dayPartOf(hour: number): DayPart {
  if (hour >= 5 && hour < 12) return "morning";
  if (hour >= 12 && hour < 19) return "afternoon";
  return "evening";
}

/** The first word of a name, to greet someone the way they are called: «María del Carmen López» → «María». */
export function firstName(fullName: string | undefined | null): string {
  return (fullName ?? "").trim().split(/\s+/)[0] ?? "";
}
