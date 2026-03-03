import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import "@shared/i18n";
import { DataTableApiProvider } from "@shared/components";
import App from "@admin/App";
import { api } from "@admin/api";
import "./app.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <DataTableApiProvider api={api}>
      <BrowserRouter basename="/admin">
        <App />
      </BrowserRouter>
    </DataTableApiProvider>
  </StrictMode>,
);
