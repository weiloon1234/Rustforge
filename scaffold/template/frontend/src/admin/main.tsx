import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { initI18n } from "@shared/i18n";
import "./app.css";

const EMPTY_SCOPES: string[] = [];

async function start() {
  await initI18n();
  const [{ default: App }, { DataTableApiProvider }, { api }, { useAuthStore }] =
    await Promise.all([
      import("@admin/App"),
      import("@shared/components"),
      import("@admin/api"),
      import("@admin/stores/auth"),
    ]);

  function Root() {
    const scopes = useAuthStore((state) => state.account?.scopes ?? EMPTY_SCOPES);

    return (
      <DataTableApiProvider api={api} scopes={scopes}>
        <BrowserRouter basename="/admin">
          <App />
        </BrowserRouter>
      </DataTableApiProvider>
    );
  }

  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <Root />
    </StrictMode>,
  );
}

void start();
