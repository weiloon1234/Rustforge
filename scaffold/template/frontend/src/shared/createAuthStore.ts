import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface Account {
  id: number;
  name: string;
  email: string | null;
}

export interface AuthState<A extends Account = Account> {
  account: A | null;
  token: string | null;
  isLoading: boolean;
  isInitialized: boolean;
  error: string | null;
  setToken: (token: string) => void;
  login: (credentials: Record<string, unknown>) => Promise<void>;
  logout: () => void;
  fetchAccount: () => Promise<void>;
  refreshToken: () => Promise<void>;
  initSession: () => Promise<void>;
}

export interface AuthConfig {
  loginEndpoint: string;    // "/api/v1/admin/auth/login"
  meEndpoint: string;       // "/api/v1/admin/auth/me"
  refreshEndpoint: string;  // "/api/v1/admin/auth/refresh"
  storageKey: string;       // "admin-auth"
}

/**
 * Factory that creates a typed auth store for any portal.
 *
 * The store uses `client_type: "web"` so the Rust backend stores the
 * refresh token in an HttpOnly cookie. The frontend only manages the
 * access token — the browser sends the cookie automatically on refresh.
 *
 * Usage:
 * ```ts
 * export const useAuthStore = createAuthStore({
 *   loginEndpoint:   "/api/v1/admin/auth/login",
 *   meEndpoint:      "/api/v1/admin/auth/me",
 *   refreshEndpoint: "/api/v1/admin/auth/refresh",
 *   storageKey:      "admin-auth",
 * });
 * ```
 *
 * For portals with extra account fields, pass a generic:
 * ```ts
 * interface MerchantAccount extends Account { companyId: number }
 * export const useAuthStore = createAuthStore<MerchantAccount>({ ... });
 * ```
 */
export function createAuthStore<A extends Account = Account>(
  config: AuthConfig,
) {
  return create<AuthState<A>>()(
    persist(
      (set, get) => ({
        account: null,
        token: null,
        isLoading: false,
        isInitialized: false,
        error: null,

        setToken: (token: string) =>
          set({ token } as Partial<AuthState<A>>),

        login: async (credentials: Record<string, unknown>) => {
          set({ isLoading: true, error: null } as Partial<AuthState<A>>);
          try {
            const res = await fetch(config.loginEndpoint, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              credentials: "include",
              body: JSON.stringify({ ...credentials, client_type: "web" }),
            });
            if (!res.ok) {
              const body = await res.json().catch(() => null);
              throw new Error(body?.message ?? "Login failed");
            }
            const { data } = await res.json();
            set({
              token: data.access_token,
              isLoading: false,
            } as Partial<AuthState<A>>);
          } catch (err) {
            set({
              error: (err as Error).message,
              isLoading: false,
            } as Partial<AuthState<A>>);
            throw err;
          }
        },

        logout: () =>
          set({ account: null, token: null } as Partial<AuthState<A>>),

        fetchAccount: async () => {
          const { token } = get();
          if (!token) return;
          set({ isLoading: true } as Partial<AuthState<A>>);
          try {
            const res = await fetch(config.meEndpoint, {
              headers: { Authorization: `Bearer ${token}` },
            });
            if (!res.ok) throw new Error("Failed to fetch account");
            const { data } = await res.json();
            set({ account: data, isLoading: false } as Partial<AuthState<A>>);
          } catch {
            set({
              account: null,
              token: null,
              isLoading: false,
            } as Partial<AuthState<A>>);
          }
        },

        refreshToken: async () => {
          const res = await fetch(config.refreshEndpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            credentials: "include",
            body: JSON.stringify({ client_type: "web" }),
          });
          if (!res.ok) {
            set({ account: null, token: null } as Partial<AuthState<A>>);
            throw new Error("Session expired");
          }
          const { data } = await res.json();
          set({ token: data.access_token } as Partial<AuthState<A>>);
        },

        initSession: async () => {
          const { token, isInitialized } = get();
          if (isInitialized) return;
          if (!token) {
            set({ isInitialized: true } as Partial<AuthState<A>>);
            return;
          }
          try {
            await get().fetchAccount();
          } catch {
            // Access token expired — try refresh
            try {
              await get().refreshToken();
              await get().fetchAccount();
            } catch {
              // Refresh also failed — session is gone
              set({ account: null, token: null } as Partial<AuthState<A>>);
            }
          }
          set({ isInitialized: true } as Partial<AuthState<A>>);
        },
      }),
      { name: config.storageKey },
    ),
  );
}
