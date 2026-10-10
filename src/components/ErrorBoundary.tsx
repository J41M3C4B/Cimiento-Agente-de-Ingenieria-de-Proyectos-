import { Component } from "react";
import type { ErrorInfo, ReactNode } from "react";
import { es } from "../i18n/es-MX";
import { Button, Card } from "./ui";

const t = es.errors.crash;

/**
 * The safety net of the whole program: when a part of the screen fails while it is drawn, the person sees what
 * happened and a way to go on, never a blank window. What was saved is in the base and is not touched.
 */
export class ErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    // for whoever looks at the console (development); the person sees the words below
    console.error(error, info.componentStack);
  }

  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <div className="grid min-h-screen place-items-center bg-canvas p-4">
        <Card className="max-w-[520px] space-y-4">
          <h1 className="text-heading font-bold">{t.title}</h1>
          <p className="text-ui text-ink-2">{t.text}</p>
          <Button variant="primary" onClick={() => window.location.reload()}>
            {t.reload}
          </Button>
        </Card>
      </div>
    );
  }
}
