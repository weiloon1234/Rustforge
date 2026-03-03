import type { AdminMeOutput } from "@admin/types/admin-auth";
import { api } from "@admin/api";
import { useAuthStore } from "@admin/stores/auth";
import { createLocalePersistence } from "@shared/createLocalePersistence";

export const adminLocalePersistence = createLocalePersistence<AdminMeOutput>({
  api,
  updateEndpoint: "auth/locale_update",
  getAccount: () => useAuthStore.getState().account,
  setAccount: (account) => useAuthStore.getState().setAccount(account),
});
