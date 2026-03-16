import type {
  AdjustableCreditType,
  BankStatus,
  CompanyBankAccountStatus,
  CompanyCryptoAccountStatus,
  CreditType,
  CryptoNetworkStatus,
  DepositMethod,
  DepositStatus,
  UserBanStatus,
  WithdrawalMethod,
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

export const OWNER_TYPE_I18N: Record<string, string> = {
  "1": "enum.owner_type.user",
  "2": "enum.owner_type.merchant",
  "3": "enum.owner_type.agent",
};

export const DEPOSIT_METHOD_I18N: Record<DepositMethod, string> = {
  "1": "enum.deposit_method.manual",
};

export const WITHDRAWAL_METHOD_I18N: Record<WithdrawalMethod, string> = {
  "1": "enum.withdrawal_method.manual",
};

export const BANK_STATUS_I18N: Record<BankStatus, string> = {
  "1": "enum.bank_status.enabled",
  "2": "enum.bank_status.disabled",
};

export const CRYPTO_NETWORK_STATUS_I18N: Record<CryptoNetworkStatus, string> = {
  "1": "enum.crypto_network_status.enabled",
  "2": "enum.crypto_network_status.disabled",
};

export const COMPANY_BANK_ACCOUNT_STATUS_I18N: Record<CompanyBankAccountStatus, string> = {
  "1": "enum.company_bank_account_status.enabled",
  "2": "enum.company_bank_account_status.disabled",
};

export const COMPANY_CRYPTO_ACCOUNT_STATUS_I18N: Record<CompanyCryptoAccountStatus, string> = {
  "1": "enum.company_crypto_account_status.enabled",
  "2": "enum.company_crypto_account_status.disabled",
};
