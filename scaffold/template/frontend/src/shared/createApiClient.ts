import axios, { type AxiosInstance, type InternalAxiosRequestConfig } from "axios";

export interface ApiClientConfig {
  /** Read the current access token (from auth store). */
  getToken: () => string | null;
  /** Attempt to refresh the session. Must throw on failure. */
  refreshAuth: () => Promise<void>;
  /** Called when refresh also fails — clear state and redirect. */
  onAuthFailure: () => void;
}

/**
 * Factory that creates an Axios instance with:
 * - Request interceptor: attaches `Authorization: Bearer <token>`
 * - Response interceptor: on 401, attempts a single token refresh then
 *   retries the original request. Concurrent 401s share one refresh call.
 */
export function createApiClient(config: ApiClientConfig): AxiosInstance {
  const api = axios.create({ withCredentials: true });

  // ── Request: attach bearer token + timezone ─────────────
  api.interceptors.request.use((req) => {
    const token = config.getToken();
    if (token) {
      req.headers.Authorization = `Bearer ${token}`;
    }
    req.headers["X-Timezone"] = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return req;
  });

  // ── Response: handle 401 → refresh → retry ─────────────
  let refreshPromise: Promise<void> | null = null;

  api.interceptors.response.use(
    (res) => res,
    async (error) => {
      const original = error.config as InternalAxiosRequestConfig & {
        _retry?: boolean;
      };

      // Only attempt refresh if there is an active session (token exists).
      // Unauthenticated requests (e.g. login) should not trigger a refresh.
      if (
        error.response?.status !== 401 ||
        original._retry ||
        !config.getToken()
      ) {
        return Promise.reject(error);
      }

      original._retry = true;

      // Deduplicate concurrent refresh calls
      if (!refreshPromise) {
        refreshPromise = config
          .refreshAuth()
          .finally(() => {
            refreshPromise = null;
          });
      }

      try {
        await refreshPromise;
      } catch {
        config.onAuthFailure();
        return Promise.reject(error);
      }

      // Retry with the new token
      const newToken = config.getToken();
      if (!newToken) {
        config.onAuthFailure();
        return Promise.reject(error);
      }

      original.headers.Authorization = `Bearer ${newToken}`;
      return api(original);
    },
  );

  return api;
}
