import type { AdminType } from "@admin/types/enums";

export interface AdminDatatableRow {
  id: number;
  username: string;
  email: string | null;
  name: string;
  admin_type: AdminType;
  abilities: string[];
  created_at: string;
  updated_at: string;
}
