import { createApiClient } from "@shared/createApiClient";
import { useAuthStore } from "@user/stores/auth";

export const api = createApiClient({
  getToken: () => useAuthStore.getState().token,
  refreshAuth: () => useAuthStore.getState().refreshToken(),
  onAuthFailure: () => {
    useAuthStore.getState().logout();
    window.location.href = "/login";
  },
});
