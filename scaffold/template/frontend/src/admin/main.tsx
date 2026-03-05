import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import "@shared/i18n";
import { DataTableApiProvider } from "@shared/components";
import App from "@admin/App";
import { api } from "@admin/api";
import { useAuthStore } from "@admin/stores/auth";
import "./app.css";

function Root() {
  const scopes = useAuthStore((state) => state.account?.scopes ?? []);

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
