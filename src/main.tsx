import React from "react";
import ReactDOM from "react-dom/client";
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from "@tanstack/react-query";
import App from "./App";
import { AccessGate } from "./features/access/AccessGate";
import { notifyIfAccessLost } from "./features/access/session";
import "@fontsource-variable/plus-jakarta-sans";
import "./index.css";

// when Rust says the session is not usable (locked, closed, a password to change), the entry screen comes back
const queryClient = new QueryClient({
  queryCache: new QueryCache({ onError: notifyIfAccessLost }),
  mutationCache: new MutationCache({ onError: notifyIfAccessLost }),
  defaultOptions: { queries: { refetchOnWindowFocus: false, retry: false } },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <AccessGate>
        <App />
      </AccessGate>
    </QueryClientProvider>
  </React.StrictMode>,
);
