import { createRealtimeStore } from "@shared/createRealtimeStore";
import { useAuthStore } from "@user/stores/auth";

export const useRealtimeStore = createRealtimeStore({
  getToken: () => useAuthStore.getState().token,
});
