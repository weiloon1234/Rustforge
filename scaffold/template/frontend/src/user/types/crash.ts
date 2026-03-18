// Generated from Rust contracts — will be overwritten by `make gen-types`

export interface CrashJoinInput {
  room_key: string;
}

export interface CrashCashoutInput {
  round_id: string;
}

export interface CrashHistoryQuery {
  room_key: string;
  limit?: number | null;
}

export interface CrashGameConfig {
  preparing_duration_secs: number;
  countdown_duration_secs: number;
  post_crash_display_secs: number;
  growth_rate: number;
  start_multiplier: number;
}

export interface CrashRoomsResponse {
  config: CrashGameConfig;
  rooms: CrashRoomOutput[];
}

export interface CrashRoomOutput {
  room_key: string;
  slug: string;
  bet_amount: string;
  fee_rate: string;
  sort_order: number;
  phase: string;
  round_id: string | null;
  phase_end_at: string | null;
  started_at: string | null;
  server_time: string | null;
  last_crash_point: string | null;
}

export interface CrashJoinOutput {
  bet_id: string;
  round_id: string;
  bet_amount: string;
  fee_amount: string;
  effective_bet: string;
  credit_1: string;
}

export interface CrashCashoutOutput {
  multiplier: string;
  payout: string;
  credit_1: string;
}

export interface CrashHistoryEntry {
  round_id: string;
  crash_point: string;
  player_count: number;
  created_at: string;
}

export interface CrashMyHistoryResponse {
  items: CrashMyBetEntry[];
  next_cursor: string | null;
}

export interface CrashMyBetEntry {
  id: string;
  room_key: string;
  bet_amount: string;
  status: string;
  cashout_multiplier: string | null;
  payout_amount: string | null;
  crash_point: string;
  created_at: string;
}
