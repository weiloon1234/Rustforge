import { create } from "zustand";

export type RealtimeStatus =
  | "disconnected"
  | "connecting"
  | "authenticating"
  | "connected"
  | "reconnecting";

export interface RealtimeState {
  status: RealtimeStatus;
  error: string | null;
  connect: () => void;
  disconnect: () => void;
  subscribe: (channel: string, room?: string) => void;
  unsubscribe: (channel: string, room?: string) => void;
  on: (event: string, callback: (data: unknown) => void) => () => void;
}

export interface RealtimeConfig {
  getToken: () => string | null;
  wsPath?: string; // default: "/ws"
  reconnectMaxRetries?: number; // default: 10
  reconnectBaseMs?: number; // default: 1000
}

export function createRealtimeStore(config: RealtimeConfig) {
  const {
    getToken,
    wsPath = "/ws",
    reconnectMaxRetries = 10,
    reconnectBaseMs = 1000,
  } = config;

  let ws: WebSocket | null = null;
  let reconnectAttempt = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let intentionalClose = false;
  const listeners = new Map<string, Set<(data: unknown) => void>>();

  function getWsUrl(): string {
    // VITE_REALTIME_URL allows explicit override (e.g. "ws://127.0.0.1:3010/ws" in dev)
    const envUrl = import.meta.env.VITE_REALTIME_URL as string | undefined;
    if (envUrl) return envUrl;
    const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${proto}//${window.location.host}${wsPath}`;
  }

  function clearReconnectTimer() {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
  }

  function emit(event: string, data: unknown) {
    const cbs = listeners.get(event);
    if (cbs) {
      for (const cb of cbs) {
        try {
          cb(data);
        } catch {
          // listener error should not break dispatch
        }
      }
    }
  }

  return create<RealtimeState>()((set, get) => ({
    status: "disconnected",
    error: null,

    connect: () => {
      const { status } = get();
      if (status === "connected" || status === "connecting" || status === "authenticating") {
        return;
      }

      intentionalClose = false;
      clearReconnectTimer();
      set({ status: "connecting", error: null });

      try {
        ws = new WebSocket(getWsUrl());
      } catch (err) {
        set({ status: "disconnected", error: (err as Error).message });
        return;
      }

      ws.onopen = () => {
        reconnectAttempt = 0;
        const token = getToken();
        if (token) {
          set({ status: "authenticating" });
          ws?.send(JSON.stringify({ op: "auth", token }));
        } else {
          // No token — connected but unauthenticated (for public channels)
          set({ status: "connected" });
        }
      };

      ws.onmessage = (event) => {
        let msg: { op: string; [key: string]: unknown };
        try {
          msg = JSON.parse(event.data as string);
        } catch {
          return;
        }

        switch (msg.op) {
          case "auth_ok":
            set({ status: "connected", error: null });
            break;
          case "auth_error":
            set({ status: "disconnected", error: (msg.message as string) ?? "Auth failed" });
            ws?.close();
            break;
          case "error":
            set({ error: (msg.message as string) ?? "Unknown error" });
            break;
          case "event": {
            const eventName = msg.event as string;
            if (eventName) {
              emit(eventName, msg.data);
            }
            break;
          }
          case "subscribed":
          case "unsubscribed":
            emit(msg.op, { channel: msg.channel, room: msg.room });
            break;
          case "pong":
            // heartbeat response — no action needed
            break;
        }
      };

      ws.onclose = () => {
        ws = null;
        if (intentionalClose) {
          set({ status: "disconnected" });
          return;
        }

        // Auto-reconnect with exponential backoff
        if (reconnectAttempt < reconnectMaxRetries) {
          const delay = Math.min(reconnectBaseMs * 2 ** reconnectAttempt, 30000);
          reconnectAttempt++;
          set({ status: "reconnecting" });
          reconnectTimer = setTimeout(() => {
            get().connect();
          }, delay);
        } else {
          set({ status: "disconnected", error: "Max reconnect attempts reached" });
        }
      };

      ws.onerror = () => {
        // onclose will fire after this — handled there
      };
    },

    disconnect: () => {
      intentionalClose = true;
      clearReconnectTimer();
      if (ws) {
        ws.close();
        ws = null;
      }
      set({ status: "disconnected", error: null });
    },

    subscribe: (channel: string, room?: string) => {
      if (!ws || ws.readyState !== WebSocket.OPEN) return;
      const msg: Record<string, string> = { op: "subscribe", channel };
      if (room) msg.room = room;
      ws.send(JSON.stringify(msg));
    },

    unsubscribe: (channel: string, room?: string) => {
      if (!ws || ws.readyState !== WebSocket.OPEN) return;
      const msg: Record<string, string> = { op: "unsubscribe", channel };
      if (room) msg.room = room;
      ws.send(JSON.stringify(msg));
    },

    on: (event: string, callback: (data: unknown) => void) => {
      if (!listeners.has(event)) {
        listeners.set(event, new Set());
      }
      listeners.get(event)!.add(callback);
      // Return unsubscribe function
      return () => {
        listeners.get(event)?.delete(callback);
      };
    },
  }));
}
