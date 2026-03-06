import type { StoreApi, UseBoundStore } from "zustand";
import { createAuthStore } from "@shared/createAuthStore";
import type { AuthState } from "@shared/createAuthStore";
import type { AdminMeOutput } from "@admin/types/admin-auth";
import type { Permission } from "@admin/types";
import {
  hasAnyPermission as hasAnyGrantedPermission,
  hasPermission as hasGrantedPermission,
} from "@shared/permissions";

type AdminAuthStore = UseBoundStore<StoreApi<AuthState<AdminMeOutput>>> & {
  hasPermission: (
    permission: Permission,
    account?: AdminMeOutput | null,
  ) => boolean;
  hasAnyPermission: (
    permissions: readonly Permission[],
    account?: AdminMeOutput | null,
  ) => boolean;
  hasAllPermissions: (
    permissions: readonly Permission[],
    account?: AdminMeOutput | null,
  ) => boolean;
};

const EMPTY_SCOPES: readonly string[] = [];

const baseStore = createAuthStore<AdminMeOutput>({
  loginEndpoint: "/api/v1/admin/auth/login",
  meEndpoint: "/api/v1/admin/auth/me",
  refreshEndpoint: "/api/v1/admin/auth/refresh",
  storageKey: "admin-auth",
});

function resolveAccount(
  account?: AdminMeOutput | null,
): AdminMeOutput | null {
  return account === undefined ? baseStore.getState().account : account;
}

function resolveScopes(account?: AdminMeOutput | null): readonly string[] {
  return resolveAccount(account)?.scopes ?? EMPTY_SCOPES;
}

function hasPermission(
  permission: Permission,
  account?: AdminMeOutput | null,
): boolean {
  return hasGrantedPermission(resolveScopes(account), permission);
}

function hasAnyPermission(
  permissions: readonly Permission[],
  account?: AdminMeOutput | null,
): boolean {
  return hasAnyGrantedPermission(resolveScopes(account), permissions);
}

function hasAllPermissions(
  permissions: readonly Permission[],
  account?: AdminMeOutput | null,
): boolean {
  return permissions.every((permission) => hasPermission(permission, account));
}

export const useAuthStore = Object.assign(baseStore, {
  hasPermission,
  hasAnyPermission,
  hasAllPermissions,
}) as AdminAuthStore;
