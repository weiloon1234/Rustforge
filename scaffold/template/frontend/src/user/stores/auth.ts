import { createAuthStore } from "@shared/createAuthStore";

export const useAuthStore = createAuthStore({
  loginEndpoint: "/api/v1/auth/login",
  meEndpoint: "/api/v1/auth/me",
  refreshEndpoint: "/api/v1/auth/refresh",
  storageKey: "user-auth",
});
