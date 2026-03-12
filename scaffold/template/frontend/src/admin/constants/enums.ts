import type {
  AdjustableCreditType,
  CreditType,
  DepositStatus,
  OwnerType,
  UserBanStatus,
  WithdrawalStatus,
} from "@admin/types";

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

export const DEPOSIT_STATUS_I18N: Record<DepositStatus, string> = {
  "1": "enum.deposit_status.pending",
  "2": "enum.deposit_status.approved",
  "3": "enum.deposit_status.rejected",
};

export const WITHDRAWAL_STATUS_I18N: Record<WithdrawalStatus, string> = {
  "1": "enum.withdrawal_status.pending",
  "2": "enum.withdrawal_status.processing",
  "3": "enum.withdrawal_status.approved",
  "4": "enum.withdrawal_status.rejected",
};

export const OWNER_TYPE_I18N: Record<OwnerType, string> = {
  "1": "enum.owner_type.user",
  "2": "enum.owner_type.merchant",
  "3": "enum.owner_type.agent",
};
