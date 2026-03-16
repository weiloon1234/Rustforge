import { createRealtimeStore } from "@shared/createRealtimeStore";
import { useAuthStore } from "@admin/stores/auth";

export const useRealtimeStore = createRealtimeStore({
  getToken: () => useAuthStore.getState().token,
});
