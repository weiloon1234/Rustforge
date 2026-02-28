import { create } from "zustand";

interface NotificationState {
  /** Map of notification keys to their pending counts. */
  counts: Record<string, number>;
  /** Get the count for a given key (returns 0 if not set). */
  getCount: (key: string) => number;
  /** Set count for a key. Call this from your polling/websocket handler. */
  setCount: (key: string, count: number) => void;
  /** Batch-set multiple counts at once. */
  setCounts: (counts: Record<string, number>) => void;
}

export const useNotificationStore = create<NotificationState>()((set, get) => ({
  counts: {},
  getCount: (key) => get().counts[key] ?? 0,
  setCount: (key, count) =>
    set((state) => ({ counts: { ...state.counts, [key]: count } })),
  setCounts: (counts) =>
    set((state) => ({ counts: { ...state.counts, ...counts } })),
}));
