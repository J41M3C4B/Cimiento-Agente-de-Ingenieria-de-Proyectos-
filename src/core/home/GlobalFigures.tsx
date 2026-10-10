import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { MODULE_META } from "../../components/modules";
import { ChartDot, FigureCard, Gauge, Waffle, gaugeShares } from "../../components/ui";
import type { GaugePart } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { careOverview } from "../../modules/care/api";
import { CARE_KEY } from "../../modules/care/CareTab";
import { FINANCE_KEY, financeGet } from "../../modules/finance/api";
import { useOverview } from "../profile/gaps";

const t = es.home.figures;
const count = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `${n < 0 ? "−" : ""}$${count(Math.abs(n))}`;

/**
 * The figures that belong to the whole institution (Inicio): how full it is and how the year goes in money. They cross
 * modules, so they live here; the figures of a single module stay in its page. Each one is its own tray of a fixed size
 * and opens the module it comes from. The third cell is `aside`: what Inicio puts next to them (the card of the institution).
 */
export function GlobalFigures({ onGo, aside }: { onGo: (page: "people" | "finance") => void; aside?: ReactNode }) {
  const overview = useOverview();
  const care = useQuery({ queryKey: CARE_KEY, queryFn: careOverview });
  const finance = useQuery({ queryKey: FINANCE_KEY, queryFn: financeGet });

  // who is served, how full the house is and what is left, as Rust composes it (ADR-033); while nobody is registered
  // the quick figure of the first start stands in, and says so («≈»)
  const o = overview.data;
  const capacity = o?.capacity ?? null;
  const indicators = care.data?.board.indicators;
  const approx = o?.people.approx ?? false;
  const served = o?.people.value ?? 0;
  // of entering and leaving, the card tells only the one that happened last, with its count of this year
  const lastLeft = indicators?.latest_movement === "discharged";
  const movedN = (lastLeft ? indicators?.discharged_this_year : indicators?.admitted_this_year) ?? 0;
  const used = o?.occupied_percent ?? 0;

  const f = finance.data?.finances;
  const balance = f?.balance_annual_mxn ?? null;
  const income = f?.income_known ? f.income_annual_mxn : null;
  const expenses = f?.expenses_annual_mxn ?? null;
  const shares = balance !== null && income !== null && expenses !== null ? gaugeShares(income, expenses) : null;
  // each color of the donut as a share of the income; the free one never goes below zero
  const percentOf = (n: number | null) => (income !== null && income > 0 && n !== null ? Math.round((Math.max(n, 0) / income) * 100) : 0);
  const known = balance !== null && income !== null && expenses !== null;
  const [hover, setHover] = useState<GaugePart | null>(null);
  const center = {
    income: { pct: income !== null && income > 0 ? 100 : 0, text: t.incomeCaption },
    spent: { pct: percentOf(expenses), text: t.spentCaption },
    left: { pct: percentOf(balance), text: t.ofBudget },
  }[hover ?? "left"];

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <FigureCard icon={MODULE_META.people.icon} title={t.people} onOpen={() => onGo("people")}>
        <div className="fig-row">
          <div className="fig-main">
            <div className="tabular whitespace-nowrap text-display font-normal tracking-tight">{approx ? `≈ ${count(served)}` : count(served)}</div>
            <div className="fig-cols">
              {capacity !== null && (
                <span className="fig-col">
                  <ChartDot tone="off" />
                  <b className="tabular">{count(o?.vacant ?? 0)}</b>
                  <span>{t.emptyPlaces}</span>
                </span>
              )}
              <span className="fig-col" title={`${lastLeft ? t.leftLabel : t.enteredLabel} · ${t.thisYear.toLowerCase()}`}>
                <span role="img" aria-label={lastLeft ? t.leftLabel : t.enteredLabel} className={lastLeft ? "text-red" : "text-green"}>
                  <Icon name={lastLeft ? "down" : "up"} size={18} strokeWidth={2.8} />
                </span>
                <b className="tabular">{count(movedN)}</b>
              </span>
            </div>
          </div>
          {capacity !== null && capacity > 0 && (
            <div className="fig-chart">
              <span className="fig-pill" style={{ left: `${Math.min(Math.max(used, 12), 88)}%` }}>
                {t.used(used)}
              </span>
              <Waffle fill aspect={2.6} places={capacity} taken={served} label={t.placesChart(count(served), count(capacity))} />
            </div>
          )}
        </div>
      </FigureCard>

      <FigureCard icon={MODULE_META.finance.icon} title={t.balance} onOpen={() => onGo("finance")}>
        <div className="fig-values">
          <div className="fig-value" onMouseEnter={() => setHover("income")} onMouseLeave={() => setHover(null)}>
            <b className="tabular">{income === null ? "—" : peso(income)}</b>
            <span>
              <ChartDot tone="off" />
              {t.income}
            </span>
          </div>
          <div className="fig-value" onMouseEnter={() => setHover("spent")} onMouseLeave={() => setHover(null)}>
            <b className="tabular">{expenses === null ? "—" : peso(expenses)}</b>
            <span>
              <ChartDot tone="ink" />
              {t.expenses}
            </span>
          </div>
          <div className="fig-value" onMouseEnter={() => setHover("left")} onMouseLeave={() => setHover(null)}>
            <b className="tabular">{balance === null ? "—" : peso(balance)}</b>
            <span>
              <ChartDot tone="brand" />
              {t.available}
            </span>
          </div>
        </div>
        <div className="my-auto">
          <Gauge
            spent={shares?.spent ?? 0}
            left={shares?.left ?? 0}
            active={known ? hover : null}
            onActive={known ? setHover : undefined}
            label={!known ? t.balanceUnknown : t.balanceChart(peso(income), peso(expenses), peso(balance))}
          >
            {known ? (
              <>
                <b>{center.pct} %</b>
                <span>{center.text}</span>
              </>
            ) : (
              <span>{es.profile.modules.financeUnknown}</span>
            )}
          </Gauge>
        </div>
      </FigureCard>

      {aside}
    </div>
  );
}
