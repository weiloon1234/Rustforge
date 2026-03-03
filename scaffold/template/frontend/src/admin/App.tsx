import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "@shared/ProtectedRoute";
import { useAuthStore } from "@admin/stores/auth";
import AdminLayout from "@admin/layouts/AdminLayout";
import LoginPage from "@admin/pages/LoginPage";
import DashboardPage from "@admin/pages/DashboardPage";
import AdminsPage from "@admin/pages/AdminsPage";
import HttpClientLogsPage from "@admin/pages/HttpClientLogsPage";
import WebhookLogsPage from "@admin/pages/WebhookLogsPage";
import PagesPage from "@admin/pages/PagesPage";
import PageEditPage from "@admin/pages/PageEditPage";

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
        <Route element={<AdminLayout />}>
          <Route index element={<DashboardPage />} />
          <Route path="/admins" element={<AdminsPage />} />
          <Route path="/pages" element={<PagesPage />} />
          <Route path="/pages/:id/edit" element={<PageEditPage />} />
          <Route path="/http-client-logs" element={<HttpClientLogsPage />} />
          <Route path="/webhook-logs" element={<WebhookLogsPage />} />
        </Route>
      </Route>
    </Routes>
  );
}
