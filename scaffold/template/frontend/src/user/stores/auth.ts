import { createAuthStore } from "@shared/createAuthStore";
import type { UserMeOutput } from "@user/types/user-auth";

export const useAuthStore = createAuthStore<UserMeOutput>({
  loginEndpoint: "/api/v1/user/auth/login",
  meEndpoint: "/api/v1/user/auth/me",
  refreshEndpoint: "/api/v1/user/auth/refresh",
  storageKey: "user-auth",
});
