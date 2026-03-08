import type { UserMeOutput } from "@user/types/user-auth";
import { api } from "@user/api";
import { useAuthStore } from "@user/stores/auth";
import { createLocalePersistence } from "@shared/createLocalePersistence";

export const userLocalePersistence = createLocalePersistence<UserMeOutput>({
  api,
  updateEndpoint: "auth/locale_update",
  getAccount: () => useAuthStore.getState().account,
  setAccount: (account) => useAuthStore.getState().setAccount(account),
});
