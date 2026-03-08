import { lazy, Suspense } from "react";
import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "@shared/ProtectedRoute";
import { useAuthStore } from "@user/stores/auth";

const UserLayout = lazy(() => import("@user/layouts/UserLayout"));
const LoginPage = lazy(() => import("@user/pages/LoginPage"));
const RegisterPage = lazy(() => import("@user/pages/RegisterPage"));
const DashboardPage = lazy(() => import("@user/pages/DashboardPage"));
const TransactionsPage = lazy(() => import("@user/pages/TransactionsPage"));
const WalletPage = lazy(() => import("@user/pages/WalletPage"));
const MePage = lazy(() => import("@user/pages/MePage"));
const MyTeamPage = lazy(() => import("@user/pages/MyTeamPage"));

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
        <Route path="/register" element={<RegisterPage />} />
        <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
          <Route element={<UserLayout />}>
            <Route index element={<DashboardPage />} />
            <Route path="/history" element={<TransactionsPage />} />
            <Route path="/wallet" element={<WalletPage />} />
            <Route path="/me" element={<MePage />} />
            <Route path="/team" element={<MyTeamPage />} />
          </Route>
        </Route>
      </Routes>
    </Suspense>
  );
}
