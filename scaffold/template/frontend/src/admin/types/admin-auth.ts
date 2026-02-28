import type { AdminType, AuthClientType } from "@admin/types/enums";

export interface AdminLoginInput {
  username: string;
  password: string;
  client_type: AuthClientType;
}

export interface AdminRefreshInput {
  client_type: AuthClientType;
  refresh_token?: string | null;
}

export interface AdminLogoutInput {
  client_type: AuthClientType;
  refresh_token?: string | null;
}

export interface AdminProfileUpdateInput {
  name: string;
  email?: string | null;
}

export interface AdminPasswordUpdateInput {
  current_password: string;
  password: string;
  password_confirmation: string;
}

export interface AdminAuthOutput {
  token_type: string;
  access_token: string;
  access_expires_at?: string | null;
  refresh_token?: string;
  scopes: string[];
}

export interface AdminMeOutput {
  id: number;
  username: string;
  email: string | null;
  name: string;
  admin_type: AdminType;
  scopes: string[];
}

export interface AdminProfileUpdateOutput {
  id: number;
  username: string;
  email: string | null;
  name: string;
  admin_type: AdminType;
}

export interface AdminPasswordUpdateOutput {
  updated: boolean;
}

export interface AdminLogoutOutput {
  revoked: boolean;
}
