import { lazy, Suspense } from "react";
import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "@shared/ProtectedRoute";
import { useAuthStore } from "@admin/stores/auth";

const AdminLayout = lazy(() => import("@admin/layouts/AdminLayout"));
const LoginPage = lazy(() => import("@admin/pages/LoginPage"));
const DashboardPage = lazy(() => import("@admin/pages/DashboardPage"));
const AdminsPage = lazy(() => import("@admin/pages/other/AdminsPage"));
const ContentPagesPage = lazy(() => import("@admin/pages/other/ContentPagesPage"));
const ContentPageEditPage = lazy(() => import("@admin/pages/other/ContentPageEditPage"));
const CountriesPage = lazy(() => import("@admin/pages/other/CountriesPage"));
const HttpClientLogsPage = lazy(() => import("@admin/pages/developer/HttpClientLogsPage"));
const WebhookLogsPage = lazy(() => import("@admin/pages/developer/WebhookLogsPage"));

function RouteFallback() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background text-muted">
      Loading...
    </div>
  );
}

export default function App() {
  return (
    <Suspense fallback={<RouteFallback />}>
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
    </Suspense>
  );
}
