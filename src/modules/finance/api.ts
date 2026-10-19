// The commands of the finance module (ADR-026, ADR-032).
import { invoke } from "@tauri-apps/api/core";
import type { Decision, FinanceInput, FinanceOutcome, FinanceView } from "../../lib/types";

/** The money changes with the staff and the fees of the people served: refresh it when they change. */
export const FINANCE_KEY = ["finance"];

export const financeGet = () => invoke<FinanceView>("finance_get");
export const financeSave = (input: FinanceInput, decision?: Decision) =>
  invoke<FinanceOutcome>("finance_save", { input, decision: decision ?? null });
