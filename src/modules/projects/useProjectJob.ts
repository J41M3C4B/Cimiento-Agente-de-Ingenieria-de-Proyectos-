import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import { projectJob } from "../../lib/tauri";
import type { AiStatus, JobKind, ProjectJob } from "../../lib/types";

/** The AI answers that need no notice to the person. */
const QUIET: AiStatus[] = ["used", "skipped"];

/**
 * What the AI is doing for a project, asked of the program (not remembered by the screen), so that work started
 * before the person changed section or reopened the project is still shown as work in progress. While something
 * runs it asks again every second and a half; when it ends, what it wrote is read from the program again.
 *
 * `ready` is false until the first answer: a screen must not start work of its own before it knows whether the AI
 * is already doing it. `notice` is how a job that ended while the person was away went wrong, if it did.
 */
export function useProjectJob(projectId: string, kinds: readonly JobKind[]) {
  return useJob(projectId, kinds, [["diagnosis", projectId], ["needs", projectId]]);
}

/** What the AI is doing for a call (the «en pocas palabras»): the slot is the call's, not a project's. */
export function useCallJob(readingId: string, kinds: readonly JobKind[]) {
  return useJob(`call:${readingId}`, kinds, [["call-reading", readingId]]);
}

/** `refetch`: the queries to read again when the work ends, because what it wrote is now in the program. */
function useJob(slot: string, kinds: readonly JobKind[], refetch: readonly (readonly string[])[]) {
  const qc = useQueryClient();
  const q = useQuery({
    queryKey: ["project-job", slot],
    queryFn: () => projectJob(slot),
    refetchInterval: (query) => (query.state.data?.running ? 1500 : false),
    // nothing is kept between visits: coming back always asks the program
    gcTime: 0,
  });
  const running = q.data?.running ?? null;
  const [ended, setEnded] = useState<{ kind: JobKind; ai: AiStatus } | null>(null);
  const before = useRef<JobKind | null>(null);
  const keys = useRef(refetch);
  keys.current = refetch;

  useEffect(() => {
    if (before.current && !running) {
      for (const queryKey of keys.current) void qc.invalidateQueries({ queryKey: [...queryKey] });
    }
    before.current = running;
  }, [running, qc]);

  useEffect(() => {
    if (q.data?.finished) setEnded(q.data.finished);
  }, [q.data]);

  const refresh = useCallback(() => qc.invalidateQueries({ queryKey: ["project-job", slot] }), [qc, slot]);
  /** The person starts something new: what ended while they were away is no longer news. */
  const dismiss = useCallback(() => {
    setEnded(null);
    qc.setQueryData<ProjectJob>(["project-job", slot], (d) => (d ? { ...d, finished: null } : d));
  }, [qc, slot]);
  // read from the answer itself, not only from what the screen kept of it: in the very render where the screen
  // learns that nothing is running it must also know that something just ended, or it would start the work again
  const latest = ended ?? q.data?.finished ?? null;
  const mine = latest && kinds.includes(latest.kind) ? latest : null;

  return {
    ready: q.isFetched,
    /** The kind of work the AI is doing for this project, if any. */
    running,
    /** One of this screen's own kinds of work is running. */
    runningHere: running !== null && kinds.includes(running),
    /** The AI finished something of this screen's while the person was away. */
    finishedAway: mine !== null,
    notice: mine && !QUIET.includes(mine.ai) ? mine.ai : null,
    refresh,
    dismiss,
  };
}
