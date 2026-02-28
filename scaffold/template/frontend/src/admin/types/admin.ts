import type { AdminType, Permission } from "@admin/types/enums";

export interface CreateAdminInput {
  username: string;
  email?: string | null;
  name: string;
  password: string;
  abilities?: Permission[];
}

export interface UpdateAdminInput {
  username?: string | null;
  email?: string | null;
  name?: string | null;
  abilities?: Permission[] | null;
}

export interface AdminOutput {
  id: number;
  username: string;
  email: string | null;
  name: string;
  admin_type: AdminType;
  abilities: string[];
  created_at: string;
  updated_at: string;
}

export interface AdminDeleteOutput {
  deleted: boolean;
}
