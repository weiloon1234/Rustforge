import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { initI18n } from "@shared/i18n";
import "./app.css";

async function start() {
  await initI18n();
  const { default: App } = await import("@user/App");

  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </StrictMode>,
  );
}

void start();
