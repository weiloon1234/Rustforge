import { useEffect } from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import type { AuthState, Account } from "@shared/createAuthStore";
import type { StoreApi, UseBoundStore } from "zustand";

interface Props {
  useAuthStore: UseBoundStore<StoreApi<AuthState<Account>>>;
  loginPath?: string;
}

/**
 * Route guard that protects child routes behind authentication.
 *
 * On first render it calls `initSession()` which:
 * 1. If a persisted token exists → validates it via `fetchAccount()`
 * 2. If access token expired → attempts `refreshToken()` then retries
 * 3. If everything fails → clears auth state
 *
 * While initializing, a loading indicator is shown.
 * Once initialized, unauthenticated users are redirected to `loginPath`.
 *
 * Usage in App.tsx:
 * ```tsx
 * <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
 *   <Route path="/*" element={<DashboardPage />} />
 * </Route>
 * ```
 */
export function ProtectedRoute({ useAuthStore, loginPath = "/login" }: Props) {
  const token = useAuthStore((s) => s.token);
  const isInitialized = useAuthStore((s) => s.isInitialized);
  const initSession = useAuthStore((s) => s.initSession);
  const location = useLocation();

  useEffect(() => {
    initSession();
  }, [initSession]);

  if (!isInitialized) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background text-foreground">
        <div className="text-muted">Loading…</div>
      </div>
    );
  }

  if (!token) {
    return <Navigate to={loginPath} state={{ from: location }} replace />;
  }

  return <Outlet />;
}
