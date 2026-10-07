import { describe, expect, it } from "vitest";
import { describeCounts, es } from "./es-MX";

// docs/08-estilo-redaccion.md: technical jargon is forbidden in the UI.
const FORBIDDEN = [
  "PII", "input", "output", "prompt", "token", "tokens", "JSON", "query", "upload",
  "dashboard", "null", "sincronizar", "parámetro", "campo obligatorio", "validación fallida",
  "error 500", "exportar", "plantilla",
];

function strings(value: unknown, out: string[] = []): string[] {
  if (typeof value === "string") out.push(value);
  else if (typeof value === "function") {
    // text builders: try them with sample numbers
    try {
      const r = (value as (n: number) => unknown)(2);
      if (typeof r === "string") out.push(r);
    } catch {
      /* not a text builder */
    }
  } else if (value && typeof value === "object") Object.values(value).forEach((v) => strings(v, out));
  return out;
}

describe("UI texts", () => {
  const all = strings(es);

  it("has texts", () => {
    expect(all.length).toBeGreaterThan(100);
  });

  it("uses no technical jargon", () => {
    const bad: string[] = [];
    for (const s of all) {
      for (const w of FORBIDDEN) {
        if (new RegExp(`(^|[^\p{L}])${w}([^\p{L}]|$)`, "iu").test(s)) bad.push(`${w}: ${s}`);
      }
    }
    expect(bad).toEqual([]);
  });

  it("describes counts in plain Spanish", () => {
    expect(describeCounts({ curp: 1 })).toBe("1 posible CURP");
    expect(describeCounts({ phone: 2, curp: 1 })).toBe("2 teléfonos y 1 posible CURP");
    expect(describeCounts({ phone: 2, curp: 1, email: 3 })).toBe("2 teléfonos, 1 posible CURP y 3 correos");
  });

  it("every issue code the app can send has a friendly text", () => {
    for (const code of [
      "name_missing", "label_missing", "negative_number", "age_range", "paying_over_count", "year_invalid",
      "amount_too_large", "population_over_capacity", "fee_estimate_ignored", "payroll_over_estimate",
      "expense_looks_like_payroll", "rfc_format", "phone_format", "email_format", "not_a_number",
    ]) {
      expect(es.issues[code], code).toBeTruthy();
    }
  });

  it("has a plain message for every way the automatic help can fail", () => {
    for (const status of ["not_configured", "offline", "busy", "budget_exhausted", "quota_reached", "key_rejected", "unavailable"]) {
      expect(es.aiNotice[status], status).toBeTruthy();
    }
  });

  it("has a plain text for every reason a call can fail and every job the AI does", () => {
    // kinds written by the pipeline in ai_usage.error_kind (AiError::kind in Rust)
    for (const kind of ["offline", "timeout", "rate_limited", "quota_reached", "auth", "refused", "truncated", "bad_output", "schema"]) {
      expect(es.ai.failures[kind], kind).toBeTruthy();
    }
    for (const task of ["conversation.turn", "diagnosis.summary", "prioritization.propose_needs", "drafting.section", "drafting.plan", "drafting.all", "call.canonical", "call.brief"]) {
      expect(es.ai.tasks[task], task).toBeTruthy();
    }
    for (const provider of ["gemini", "anthropic"]) {
      expect(es.ai.providers[provider], provider).toBeTruthy();
    }
  });

  it("names every project stage", () => {
    for (const stage of ["PROFILE", "DIAGNOSIS", "PRIORITIZATION", "CALL_SELECTION", "DRAFTING", "REVIEW", "READY"]) {
      expect(es.stages[stage], stage).toBeTruthy();
    }
  });
});
