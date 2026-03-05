import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "@shared/ProtectedRoute";
import { useAuthStore } from "@admin/stores/auth";
import AdminLayout from "@admin/layouts/AdminLayout";
import LoginPage from "@admin/pages/LoginPage";
import DashboardPage from "@admin/pages/DashboardPage";
import AdminsPage from "@admin/pages/other/AdminsPage";
import HttpClientLogsPage from "@admin/pages/developer/HttpClientLogsPage";
import WebhookLogsPage from "@admin/pages/developer/WebhookLogsPage";
import ContentPagesPage from "@admin/pages/other/ContentPagesPage";
import ContentPageEditPage from "@admin/pages/other/ContentPageEditPage";
import CountriesPage from "@admin/pages/other/CountriesPage";

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
        <Route element={<AdminLayout />}>
          <Route index element={<DashboardPage />} />
          <Route path="/other/admins" element={<AdminsPage />} />
          <Route path="/other/content-pages" element={<ContentPagesPage />} />
          <Route path="/other/content-pages/:id/edit" element={<ContentPageEditPage />} />
          <Route path="/other/countries" element={<CountriesPage />} />
          <Route path="/developer/http-client-logs" element={<HttpClientLogsPage />} />
          <Route path="/developer/webhook-logs" element={<WebhookLogsPage />} />
        </Route>
      </Route>
    </Routes>
  );
}
