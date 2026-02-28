import { createAuthStore } from "@shared/createAuthStore";
import type { AdminMeOutput } from "@admin/types/admin-auth";

export const useAuthStore = createAuthStore<AdminMeOutput>({
  loginEndpoint: "/api/v1/admin/auth/login",
  meEndpoint: "/api/v1/admin/auth/me",
  refreshEndpoint: "/api/v1/admin/auth/refresh",
  storageKey: "admin-auth",
});
