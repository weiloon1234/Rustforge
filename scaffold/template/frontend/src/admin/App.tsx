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
const ManageUsersPage = lazy(() => import("@admin/pages/user/ManageUsersPage"));
const UserHierarchyPage = lazy(() => import("@admin/pages/user/UserHierarchyPage"));
const IntroducerChangesPage = lazy(() => import("@admin/pages/user/IntroducerChangesPage"));
const AdjustCreditsPage = lazy(() => import("@admin/pages/user/AdjustCreditsPage"));
const AuditLogsPage = lazy(() => import("@admin/pages/other/AuditLogsPage"));
const DepositsPage = lazy(() => import("@admin/pages/finance/DepositsPage"));
const WithdrawalsPage = lazy(() => import("@admin/pages/finance/WithdrawalsPage"));
const BanksPage = lazy(() => import("@admin/pages/finance/BanksPage"));
const CryptoNetworksPage = lazy(() => import("@admin/pages/finance/CryptoNetworksPage"));
const CompanyBankAccountsPage = lazy(() => import("@admin/pages/finance/CompanyBankAccountsPage"));
const CompanyCryptoAccountsPage = lazy(() => import("@admin/pages/finance/CompanyCryptoAccountsPage"));
const HttpClientLogsPage = lazy(() => import("@admin/pages/developer/HttpClientLogsPage"));
const WebhookLogsPage = lazy(() => import("@admin/pages/developer/WebhookLogsPage"));
const SqlProfilerRequestsPage = lazy(() => import("@admin/pages/developer/SqlProfilerRequestsPage"));
const SqlProfilerQueriesPage = lazy(() => import("@admin/pages/developer/SqlProfilerQueriesPage"));
const LogViewerPage = lazy(() => import("@admin/pages/developer/LogViewerPage"));

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
            <Route path="/other/audit-logs" element={<AuditLogsPage />} />
            <Route path="/finance/deposits" element={<DepositsPage />} />
            <Route path="/finance/withdrawals" element={<WithdrawalsPage />} />
            <Route path="/finance/banks" element={<BanksPage />} />
            <Route path="/finance/crypto-networks" element={<CryptoNetworksPage />} />
            <Route path="/finance/company-bank-accounts" element={<CompanyBankAccountsPage />} />
            <Route path="/finance/company-crypto-accounts" element={<CompanyCryptoAccountsPage />} />
            <Route path="/user/manage" element={<ManageUsersPage />} />
            <Route path="/user/hierarchy" element={<UserHierarchyPage />} />
            <Route path="/user/adjust-credits" element={<AdjustCreditsPage />} />
            <Route path="/user/introducer-changes" element={<IntroducerChangesPage />} />
            <Route path="/developer/http-client-logs" element={<HttpClientLogsPage />} />
            <Route path="/developer/webhook-logs" element={<WebhookLogsPage />} />
            <Route path="/developer/sql-profiler-requests" element={<SqlProfilerRequestsPage />} />
            <Route path="/developer/sql-profiler-queries" element={<SqlProfilerQueriesPage />} />
            <Route path="/developer/log-viewer" element={<LogViewerPage />} />
          </Route>
        </Route>
      </Routes>
    </Suspense>
  );
}
