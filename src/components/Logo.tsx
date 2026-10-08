import { es } from "../i18n/es-MX";

/**
 * The SociAI mark: an S drawn as one line that joins two points (people), with a spark (the help from the AI).
 * It takes the color of the text around it, so it works on the brand tile, on the blue panel and on white.
 */
export function SociaiMark({ size = 28, className = "" }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden="true" className={className}>
      <path
        d="M22.5 10.2C21.2 8 18.8 6.8 16 6.8c-3.5 0-6 1.9-6 4.6 0 6.2 12 3.3 12 9.5 0 2.7-2.6 4.6-6 4.6-3 0-5.5-1.2-6.9-3.4"
        stroke="currentColor"
        strokeWidth="3"
        strokeLinecap="round"
      />
      <circle cx="25.2" cy="11.6" r="2.7" fill="currentColor" />
      <circle cx="6.6" cy="22.2" r="2.7" fill="currentColor" />
      <path d="M26 2.4l1 2.6 2.6 1-2.6 1-1 2.6-1-2.6-2.6-1 2.6-1z" fill="currentColor" />
    </svg>
  );
}

/** The mark on its blue tile, for the corner of the program. */
export function SociaiTile({ className = "" }: { className?: string }) {
  return (
    <span className={`brand-tile h-ctl w-ctl ${className}`} title={es.app.name}>
      <SociaiMark size={26} />
    </span>
  );
}

/** The logo: the mark and the name, «Soci» heavy and «AI» light. On the blue panel it is white. */
export function SociaiLogo({ size = "md", className = "" }: { size?: "md" | "lg"; className?: string }) {
  const big = size === "lg";
  return (
    <span className={`inline-flex items-center gap-3 ${className}`} aria-label={es.app.name} role="img">
      <SociaiMark size={big ? 44 : 34} />
      <span aria-hidden="true" className={`${big ? "text-title" : "text-subtitle"} leading-none tracking-tight`}>
        <b className="font-extrabold">Soci</b>
        <span className="font-medium">AI</span>
      </span>
    </span>
  );
}
