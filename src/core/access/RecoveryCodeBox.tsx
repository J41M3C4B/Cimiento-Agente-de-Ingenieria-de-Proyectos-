import { useState } from "react";
import { Icon } from "../../components/icons";
import { Button, Inset, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";

const t = es.access;

/** The recovery code, big and easy to copy by hand: it shows once, so it says so. Used by the first account and by the panel. */
export function RecoveryCodeBox({ code }: { code: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <Inset className="flex flex-col items-center gap-4 !py-6 text-center">
      <Tag tone="amber" icon="warn">
        {t.codeOnce}
      </Tag>
      <span className="tabular select-all text-subtitle font-extrabold tracking-widest whitespace-nowrap">{code}</span>
      <Button
        size="sm"
        variant="secondary"
        onClick={() => {
          void navigator.clipboard?.writeText(code);
          setCopied(true);
          window.setTimeout(() => setCopied(false), 2500);
        }}
      >
        <Icon name={copied ? "check" : "file"} size={16} />
        {copied ? t.codeCopied : t.copyCode}
      </Button>
    </Inset>
  );
}
