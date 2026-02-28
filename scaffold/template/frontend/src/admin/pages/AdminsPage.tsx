import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Pencil, Trash2, ArrowUp, ArrowDown, ArrowUpDown } from "lucide-react";
import type { AdminDeleteOutput, AdminDatatableRow, AdminType, Permission } from "@admin/types";
import type { ApiResponse } from "@shared/types";
import {
  Checkbox,
  DataTable,
  useAutoForm,
  useModalStore,
  alertConfirm,
  alertSuccess,
  alertError,
  formatDateTime,
} from "@shared/components";
import type { DataTableSortState } from "@shared/components";
import { api } from "@admin/api";

const TYPE_COLORS: Record<AdminType, string> = {
  developer: "bg-purple-100 text-purple-700",
  superadmin: "bg-amber-100 text-amber-700",
  admin: "bg-blue-100 text-blue-700",
};

const ALL_PERMISSIONS: Permission[] = ["admin.read", "admin.manage"];

const PERMISSION_LABELS: Record<Permission, string> = {
  "admin.read": "Read Admins",
  "admin.manage": "Manage Admins",
};

function TypeBadge({ type }: { type: AdminType }) {
  return (
    <span
      className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${TYPE_COLORS[type] ?? "bg-gray-100 text-gray-700"}`}
    >
      {type}
    </span>
  );
}

function PermissionBadges({ abilities }: { abilities: string[] }) {
  const { t } = useTranslation();
  if (abilities.includes("*")) {
    return (
      <span className="inline-block rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-700">
        {t("All permissions")}
      </span>
    );
  }
  return (
    <div className="flex flex-wrap gap-1">
      {abilities.map((a) => (
        <span
          key={a}
          className="inline-block rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-600"
        >
          {a}
        </span>
      ))}
    </div>
  );
}

function PermissionCheckboxes({
  abilities,
  onChange,
}: {
  abilities: string[];
  onChange: (next: string[]) => void;
}) {
  const { t } = useTranslation();
  return (
    <fieldset className="space-y-2">
      <legend className="text-sm font-medium text-foreground">{t("Permissions")}</legend>
      <div className="flex flex-wrap gap-x-6 gap-y-1">
        {ALL_PERMISSIONS.map((perm) => (
          <Checkbox
            key={perm}
            label={t(PERMISSION_LABELS[perm])}
            checked={abilities.includes(perm)}
            onChange={(e) => {
              if (e.target.checked) {
                onChange([...abilities, perm]);
              } else {
                onChange(abilities.filter((a) => a !== perm));
              }
            }}
          />
        ))}
      </div>
    </fieldset>
  );
}

function CreateAdminForm({ onCreated }: { onCreated: () => void }) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);
  const [abilities, setAbilities] = useState<string[]>([]);

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "/api/v1/admin/admins",
    method: "post",
    extraPayload: { abilities },
    fields: [
      { name: "username", type: "text", label: t("Username"), placeholder: t("Enter username"), required: true },
      { name: "name", type: "text", label: t("Name"), placeholder: t("Enter full name"), required: true },
      { name: "email", type: "email", label: t("Email"), placeholder: t("Enter email"), required: false },
      { name: "password", type: "password", label: t("Password"), placeholder: t("Enter password"), required: true },
    ],
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Admin created") });
      onCreated();
    },
  });

  return (
    <form onSubmit={submit} className="space-y-4">
      {errors.general && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {errors.general}
        </p>
      )}
      {form}
      <PermissionCheckboxes abilities={abilities} onChange={setAbilities} />
      <div className="flex justify-end gap-2 pt-2">
        <button type="button" onClick={() => close()} className="rf-modal-btn-secondary">
          {t("Cancel")}
        </button>
        <button type="submit" disabled={busy} className="rf-modal-btn-primary">
          {busy ? t("Creating…") : t("Create")}
        </button>
      </div>
    </form>
  );
}

function EditAdminForm({
  admin,
  onUpdated,
}: {
  admin: AdminDatatableRow;
  onUpdated: () => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);
  const isNormalAdmin = admin.admin_type === "admin";
  const [abilities, setAbilities] = useState<string[]>(
    admin.abilities.filter((a) => a !== "*"),
  );

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: `/api/v1/admin/admins/${admin.id}`,
    method: "patch",
    extraPayload: isNormalAdmin ? { abilities } : {},
    fields: [
      { name: "username", type: "text", label: t("Username"), placeholder: t("Enter username"), required: true },
      { name: "name", type: "text", label: t("Name"), placeholder: t("Enter full name"), required: true },
      { name: "email", type: "email", label: t("Email"), placeholder: t("Enter email"), required: false },
    ],
    defaults: {
      username: admin.username,
      name: admin.name,
      email: admin.email ?? "",
    },
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Admin updated") });
      onUpdated();
    },
  });

  return (
    <form onSubmit={submit} className="space-y-4">
      {errors.general && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {errors.general}
        </p>
      )}
      {form}
      {isNormalAdmin && (
        <PermissionCheckboxes abilities={abilities} onChange={setAbilities} />
      )}
      <div className="flex justify-end gap-2 pt-2">
        <button type="button" onClick={() => close()} className="rf-modal-btn-secondary">
          {t("Cancel")}
        </button>
        <button type="submit" disabled={busy} className="rf-modal-btn-primary">
          {busy ? t("Saving…") : t("Save")}
        </button>
      </div>
    </form>
  );
}

export default function AdminsPage() {
  const { t } = useTranslation();

  const handleCreate = (refresh: () => void) => {
    useModalStore.getState().open({
      title: t("Create Admin"),
      size: "lg",
      content: <CreateAdminForm onCreated={refresh} />,
    });
  };

  const handleEdit = (admin: AdminDatatableRow, refresh: () => void) => {
    useModalStore.getState().open({
      title: t("Edit Admin"),
      size: "lg",
      content: <EditAdminForm admin={admin} onUpdated={refresh} />,
    });
  };

  const handleDelete = async (admin: AdminDatatableRow, refresh: () => void) => {
    await alertConfirm({
      title: t("Delete Admin"),
      message: t('Are you sure you want to delete ":username"?', { username: admin.username }),
      confirmText: t("Delete"),
      callback: async (result) => {
        if (result.isConfirmed) {
          try {
            await api.delete<ApiResponse<AdminDeleteOutput>>(
              `/api/v1/admin/admins/${admin.id}`,
            );
            alertSuccess({ title: t("Deleted"), message: t("Admin deleted") });
            refresh();
          } catch {
            alertError({ title: t("Error"), message: t("Failed to delete admin.") });
          }
        }
      },
    });
  };

  return (
    <DataTable<AdminDatatableRow>
      url="/api/v1/admin/datatable/admin/query"
      api={api}
      perPage={30}
      hiddenColumns={["id", "password", "updated_at", "deleted_at"]}
      prependColumns={({ column, direction, handleSort }: DataTableSortState) => (
        <>
          <th
            className="w-12 px-4 py-3 font-medium text-muted cursor-pointer select-none"
            onClick={() => handleSort("id")}
          >
            <span className="inline-flex items-center gap-1">
              #
              {column === "id" && direction === "asc" && <ArrowUp size={14} />}
              {column === "id" && direction === "desc" && <ArrowDown size={14} />}
              {column !== "id" && <ArrowUpDown size={14} className="opacity-30" />}
            </span>
          </th>
          <th className="px-4 py-3 font-medium text-muted">{t("Actions")}</th>
        </>
      )}
      renderPrependCells={(admin, index, refresh) => (
        <>
          <td className="px-4 py-3 tabular-nums text-muted">{index + 1}</td>
          <td className="px-4 py-3">
            <div className="flex gap-1">
              <button
                onClick={() => handleEdit(admin, refresh)}
                className="rounded-lg p-1.5 text-muted transition hover:bg-surface-hover hover:text-foreground"
                title={t("Edit")}
              >
                <Pencil size={16} />
              </button>
              {admin.admin_type === "admin" && (
                <button
                  onClick={() => handleDelete(admin, refresh)}
                  className="rounded-lg p-1.5 text-muted transition hover:bg-red-50 hover:text-red-600"
                  title={t("Delete")}
                >
                  <Trash2 size={16} />
                </button>
              )}
            </div>
          </td>
        </>
      )}
      columnRenderers={{
        username: (v) => <td key="username" className="px-4 py-3 font-medium text-foreground">{String(v)}</td>,
        email: (v) => <td key="email" className="px-4 py-3 text-muted">{String(v ?? "—")}</td>,
        admin_type: (_, record) => <td key="admin_type" className="px-4 py-3"><TypeBadge type={record.admin_type} /></td>,
        abilities: (_, record) => <td key="abilities" className="px-4 py-3"><PermissionBadges abilities={record.abilities} /></td>,
        created_at: (v) => <td key="created_at" className="px-4 py-3 tabular-nums text-muted">{formatDateTime(v as string)}</td>,
      }}
      rowKey={(admin) => String(admin.id)}
      header={(refresh) => (
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-foreground">{t("Admins")}</h1>
            <p className="mt-1 text-sm text-muted">{t("Manage administrator accounts")}</p>
          </div>
          <button
            onClick={() => handleCreate(refresh)}
            className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-3 py-2 text-sm font-medium text-white transition hover:bg-primary/90"
          >
            <Plus size={16} />
            {t("Create Admin")}
          </button>
        </div>
      )}
    />
  );
}
