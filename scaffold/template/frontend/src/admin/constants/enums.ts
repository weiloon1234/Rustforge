import type { AdjustableCreditType, CreditType, UserBanStatus } from "@admin/types";

export const CREDIT_TYPE_I18N: Record<CreditType, string> = {
  "1": "enum.credit_type.credit1",
  "2": "enum.credit_type.credit2",
};

export const ADJUSTABLE_CREDIT_TYPE_I18N: Record<AdjustableCreditType, string> = {
  "1": "enum.adjustable_credit_type.credit1",
};

export const BAN_STATUS_I18N: Record<UserBanStatus, string> = {
  "0": "enum.user_ban_status.no",
  "1": "enum.user_ban_status.yes",
};
