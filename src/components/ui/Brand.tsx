const NAME = "SociAI";

/** The two blobs of the S; the second is the first turned half a turn. */
const MARK = (
  <>
    <path
      className="lg-mark"
      d="M49.84,10.64c-2.47,3.48-5.69,6.11-9.23,8.41-8.58,5.62-16.96,6.07-20.84,14.06-.68,1.53-.98,3.2-.86,4.89.08,1.37.53,2.72.68,4.04.26,1.79-.53,3.62-1.91,4.79-4.97,4.01-10.45-.81-13.49-4.9-5.32-6.64-5.25-16.37-1.66-23.26,3.08-6.19,8.07-10.89,14.28-13.95C27.12-.3,39.89-1.19,47.76,1.46c3.43,1.57,4.33,6.06,2.12,9.12l-.04.07Z"
    />
    <path
      className="lg-mark"
      d="M1.24,66.67c2.47-3.48,5.69-6.11,9.23-8.41,8.58-5.62,16.96-6.07,20.84-14.06.68-1.53.98-3.2.86-4.89-.08-1.37-.53-2.72-.68-4.04-.26-1.79.53-3.62,1.91-4.79,4.97-4.01,10.45.81,13.49,4.9,5.32,6.64,5.25,16.37,1.66,23.26-3.08,6.19-8.07,10.89-14.28,13.95-10.3,5.03-23.08,5.91-30.94,3.26-3.43-1.57-4.33-6.06-2.12-9.12l.04-.07Z"
    />
  </>
);

/**
 * The logo of SociAI (docs/13 §7.1): the S and the name. The colors come from the `--logo-*` tokens and follow the
 * theme, so the same piece works on a white or a dark background. `inverse` is for surfaces painted with `ink`
 * (dark in the light theme, light in the dark one). Size it with a height (`h-8`); the width follows.
 * The name is text in the program's own font, so it only needs the font the app already carries.
 */
export function Logo({ inverse, className = "" }: { inverse?: boolean; className?: string }) {
  return (
    <svg className={`logo ${inverse ? "logo--inverse" : ""} ${className}`} viewBox="-1 -1 212.7 79.3" role="img" aria-label={NAME}>
      {MARK}
      <text className="lg-word" transform="translate(63.83 51.77)">
        <tspan>Soci</tspan>
        <tspan className="lg-ai" x="98.16" y="0">
          AI
        </tspan>
      </text>
    </svg>
  );
}

/** Just the S, for the small places (a round icon, the tab). */
export function LogoMark({ inverse, className = "" }: { inverse?: boolean; className?: string }) {
  return (
    <svg className={`logo ${inverse ? "logo--inverse" : ""} ${className}`} viewBox="-1 -1 53.1 79.3" role="img" aria-label={NAME}>
      {MARK}
    </svg>
  );
}
