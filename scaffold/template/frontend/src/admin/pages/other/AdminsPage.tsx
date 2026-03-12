import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Pencil, Trash2 } from "lucide-react";
import type {
  AdminMeOutput,
  AdminDatatableSummaryOutput,
  AdminDeleteOutput,
  AdminDatatableRow,
  AdminType,
  Permission,
  PermissionMeta,
} from "@admin/types";
import { ADMIN_TYPE, PERMISSION, PERMISSIONS, PERMISSION_META } from "@admin/types";
import type { ApiResponse } from "@shared/types";
import {
  Button,
  DataTable,
  type DataTableCellContext,
  useAutoForm,
  useModalStore,
  alertConfirm,
  alertSuccess,
  alertError,
  formatDateTime,
} from "@shared/components";
import type { DataTablePostCallEvent } from "@shared/components";
import { useAuthStore } from "@admin/stores/auth";
import { api } from "@admin/api";

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

const TYPE_COLORS: Record<AdminType, string> = {
  developer: "bg-purple-100 text-purple-700",
  superadmin: "bg-amber-100 text-amber-700",
  admin: "bg-blue-100 text-blue-700",
};

const ADMIN_PERMISSION_META = PERMISSION_META.filter(
  (meta) => meta.guard.toLowerCase() === "admin",
);
const RESTRICTED_ADMIN_ASSIGNMENT_PERMISSIONS: readonly Permission[] = [
  PERMISSION.ADMIN_READ,
  PERMISSION.ADMIN_MANAGE,
];

const ENABLE_SUMMARY_CARDS = true;

function resolvePermissionLabel(
  t: (key: string, options?: Record<string, unknown>) => string,
  permissionKey: string,
  fallbackLabel?: string,
): string {
  const translatedByKey = t(permissionKey);
  if (translatedByKey !== permissionKey) return translatedByKey;

  if (!fallbackLabel) return permissionKey;

  const translatedByLabel = t(fallbackLabel);
  if (translatedByLabel !== fallbackLabel) return translatedByLabel;
  return fallbackLabel;
}

function TypeBadge({ type }: { type: AdminType }) {
  return (
    <span
      className={`inline-flex min-h-6 items-center rounded-full px-2 py-0.5 text-xs font-medium ${TYPE_COLORS[type] ?? "bg-gray-100 text-gray-700"}`}
    >
      {type}
    </span>
  );
}

function PermissionSummary({ admin }: { admin: AdminDatatableRow }) {
  const { t } = useTranslation();
  if (admin.admin_type !== ADMIN_TYPE.ADMIN || admin.abilities.includes("*")) {
    return (
      <span className="inline-flex min-h-6 items-center rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-700">
        {t("All permissions")}
      </span>
    );
  }

  const abilities = admin.abilities;
  const openPermissionModal = () => {
    useModalStore.getState().open({
      title: t("Permissions"),
      size: "md",
      content: (
        <div className="space-y-3">
          <p className="text-sm text-muted">
            {t("Permissions for :username", { username: admin.username })}
          </p>
          {abilities.length === 0 ? (
            <p className="text-sm text-muted">{t("No permissions assigned")}</p>
          ) : (
            <div className="space-y-2">
              {abilities.map((ability) => {
                const meta = ADMIN_PERMISSION_META.find(
                  (item) => item.key === ability,
                );
                const displayLabel = resolvePermissionLabel(
                  t,
                  ability,
                  meta?.label,
                );
                return (
                  <div
                    key={ability}
                    className="rounded-lg border border-border bg-surface px-3 py-2"
                  >
                    <p className="text-sm font-medium">{displayLabel}</p>
                    <p className="text-xs text-muted">{ability}</p>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      ),
      footer: (
        <Button
          type="button"
          onClick={() => useModalStore.getState().close()}
          variant="secondary"
        >
          {t("Close")}
        </Button>
      ),
    });
  };

  return (
    <button
      type="button"
      className="inline-flex min-h-6 items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-700 transition-colors hover:bg-gray-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
      onClick={openPermissionModal}
    >
      {t(":count Permissions", { count: abilities.length })}
    </button>
  );
}

function CreateAdminForm({
  onCreated,
  formId,
  availablePermissions,
  onBusyChange,
}: {
  onCreated: () => void;
  formId: string;
  availablePermissions: readonly PermissionMeta[];
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  const { submit, busy, form } = useAutoForm(api, {
    url: "admins",
    method: "post",
    fields: [
      {
        name: "username",
        type: "text",
        label: t("Username"),
        placeholder: t("Enter username"),
        required: true,
      },
      {
        name: "name",
        type: "text",
        label: t("Name"),
        placeholder: t("Enter full name"),
        required: true,
      },
      {
        name: "email",
        type: "email",
        label: t("Email"),
        placeholder: t("Enter email"),
        required: false,
      },
      {
        name: "password",
        type: "password",
        label: t("Password"),
        placeholder: t("Enter password"),
        required: true,
      },
      ...(availablePermissions.length > 0 ? [{
        name: "abilities",
        type: "checkboxGroup" as const,
        label: t("Permissions"),
        options: availablePermissions.map((meta) => ({
          value: meta.key,
          label: resolvePermissionLabel(t, meta.key, meta.label),
        })),
        columns: 2,
      }] : []),
    ],
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Admin created") });
      onCreated();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to create admin.")),
      });
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit}>
      {form}
    </form>
  );
}

function EditAdminForm({
  admin,
  onUpdated,
  formId,
  availablePermissions,
  onBusyChange,
}: {
  admin: AdminDatatableRow;
  onUpdated: () => void;
  formId: string;
  availablePermissions: readonly PermissionMeta[];
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);
  const isNormalAdmin = admin.admin_type === ADMIN_TYPE.ADMIN;
  const currentPermissions = admin.abilities.filter(
    (value): value is Permission => PERMISSIONS.includes(value as Permission),
  );
  const assignablePermissionKeys = new Set(
    availablePermissions.map((meta) => meta.key),
  );
  const canEditAbilities =
    isNormalAdmin &&
    currentPermissions.every((permission) =>
      assignablePermissionKeys.has(permission),
    );

  const { submit, busy, form } = useAutoForm(api, {
    url: `admins/${admin.id}`,
    method: "patch",
    extraPayload: { id: admin.id },
    fields: [
      {
        name: "username",
        type: "text",
        label: t("Username"),
        placeholder: t("Enter username"),
        required: true,
      },
      {
        name: "name",
        type: "text",
        label: t("Name"),
        placeholder: t("Enter full name"),
        required: true,
      },
      {
        name: "email",
        type: "email",
        label: t("Email"),
        placeholder: t("Enter email"),
        required: false,
      },
      {
        name: "password",
        type: "password",
        label: t("Password"),
        placeholder: t("Enter password"),
        required: false,
        notes: t("Leave blank to keep current password"),
      },
      ...(canEditAbilities && availablePermissions.length > 0 ? [{
        name: "abilities",
        type: "checkboxGroup" as const,
        label: t("Permissions"),
        options: availablePermissions.map((meta) => ({
          value: meta.key,
          label: resolvePermissionLabel(t, meta.key, meta.label),
        })),
        columns: 2,
      }] : []),
    ],
    defaults: {
      username: admin.username,
      name: admin.name,
      email: admin.email ?? "",
      ...(canEditAbilities ? {
        abilities: JSON.stringify(currentPermissions.filter((p) => assignablePermissionKeys.has(p))),
      } : {}),
    },
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Admin updated") });
      onUpdated();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to update admin.")),
      });
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit}>
      {form}
    </form>
  );
}

function isPrivilegedAdminType(adminType: AdminType | null | undefined): boolean {
  return (
    adminType === ADMIN_TYPE.DEVELOPER || adminType === ADMIN_TYPE.SUPERADMIN
  );
}

function canManageAdminAccounts(account: AdminMeOutput | null): boolean {
  if (!account) return false;
  if (isPrivilegedAdminType(account.admin_type)) return true;
  return useAuthStore.hasPermission(PERMISSION.ADMIN_MANAGE, account);
}

function resolveAssignableAdminPermissions(
  account: AdminMeOutput | null,
): readonly PermissionMeta[] {
  if (!account) return [];
  if (isPrivilegedAdminType(account.admin_type)) {
    return ADMIN_PERMISSION_META;
  }

  return ADMIN_PERMISSION_META.filter(
    (meta) =>
      useAuthStore.hasPermission(meta.key, account) &&
      !RESTRICTED_ADMIN_ASSIGNMENT_PERMISSIONS.includes(meta.key),
  );
}

export default function AdminsPage() {
  const { t } = useTranslation();
  const account = useAuthStore((state) => state.account);
  const [summary, setSummary] = useState<AdminDatatableSummaryOutput | null>(
    null,
  );
  const [deletingAdminId, setDeletingAdminId] = useState<string | null>(null);
  const summaryRequestId = useRef(0);
  const canManageAdmins = useMemo(
    () => canManageAdminAccounts(account),
    [account],
  );
  const assignableAdminPermissions = useMemo(
    () => resolveAssignableAdminPermissions(account),
    [account],
  );

  const handleDatatablePostCall = (
    event: DataTablePostCallEvent<AdminDatatableRow>,
  ) => {
    if (!ENABLE_SUMMARY_CARDS) return;
    if (!event.response || event.error) {
      setSummary(null);
      return;
    }

    const requestId = ++summaryRequestId.current;
    const payload: Record<string, unknown> = {
      base: {
        include_meta: false,
      },
      ...event.filters.applied,
    };

    void api
      .post<ApiResponse<AdminDatatableSummaryOutput>>(
        "datatable/admin/summary",
        payload,
      )
      .then((res) => {
        if (summaryRequestId.current !== requestId) return;
        setSummary(res.data?.data ?? null);
      })
      .catch(() => {
        if (summaryRequestId.current !== requestId) return;
        setSummary(null);
      });
  };

  const handleCreate = (refresh: () => void) => {
    const formId = `admin-create-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button
          type="button"
          onClick={() => useModalStore.getState().close()}
          variant="secondary"
          disabled={busy}
        >
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Creating…") : t("Create")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Create Admin"),
      size: "lg",
      content: (
        <CreateAdminForm
          onCreated={refresh}
          formId={formId}
          availablePermissions={assignableAdminPermissions}
          onBusyChange={(busy) => {
            if (!modalId) return;
            useModalStore
              .getState()
              .update(modalId, { footer: renderFooter(busy) });
          }}
        />
      ),
      footer: renderFooter(false),
    });
  };

  const handleEdit = (admin: AdminDatatableRow, refresh: () => void) => {
    const formId = `admin-edit-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button
          type="button"
          onClick={() => useModalStore.getState().close()}
          variant="secondary"
          disabled={busy}
        >
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Saving…") : t("Save")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Edit Admin"),
      size: "lg",
      content: (
        <EditAdminForm
          admin={admin}
          onUpdated={refresh}
          formId={formId}
          availablePermissions={assignableAdminPermissions}
          onBusyChange={(busy) => {
            if (!modalId) return;
            useModalStore
              .getState()
              .update(modalId, { footer: renderFooter(busy) });
          }}
        />
      ),
      footer: renderFooter(false),
    });
  };

  const handleDelete = async (
    admin: AdminDatatableRow,
    refresh: () => void,
  ) => {
    await alertConfirm({
      title: t("Delete Admin"),
      message: t('Are you sure you want to delete ":username"?', {
        username: admin.username,
      }),
      confirmText: t("Delete"),
      callback: async (result) => {
        if (result.isConfirmed) {
          if (deletingAdminId === admin.id) return;
          setDeletingAdminId(admin.id);
          try {
            await api.delete<ApiResponse<AdminDeleteOutput>>(
              `admins/${admin.id}`,
            );
            alertSuccess({ title: t("Deleted"), message: t("Admin deleted") });
            refresh();
          } catch {
            alertError({
              title: t("Error"),
              message: t("Failed to delete admin."),
            });
          } finally {
            setDeletingAdminId(null);
          }
        }
      },
    });
  };

  return (
    <DataTable<AdminDatatableRow>
      url="datatable/admin/query"
      title={t("Admins")}
      subtitle={t("Manage administrator accounts")}
      headerActions={
        canManageAdmins
          ? (refresh) => (
              <Button
                onClick={() => handleCreate(refresh)}
                variant="primary"
                size="sm"
              >
                <Plus size={16} />
                {t("Create Admin")}
              </Button>
            )
          : undefined
      }
      headerContent={
        ENABLE_SUMMARY_CARDS && summary ? (
          <div className="grid gap-2 sm:grid-cols-4">
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Filtered Total")}</p>
              <p className="font-semibold">
                {summary.total_admin_counts ?? summary.total_filtered}
              </p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Developers")}</p>
              <p className="font-semibold">{summary.developer_count}</p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Super Admins")}</p>
              <p className="font-semibold">{summary.superadmin_count}</p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Admins")}</p>
              <p className="font-semibold">{summary.admin_count}</p>
            </div>
          </div>
        ) : undefined
      }
      columns={[
        ...(canManageAdmins
          ? [
              {
                key: "actions",
                label: t("Actions"),
                sortable: false,
                render: (
                  admin: AdminDatatableRow,
                  ctx: DataTableCellContext<AdminDatatableRow>,
                ) => {
                  const deleting = deletingAdminId === admin.id;
                  const isSelf = account?.id === admin.id;
                  return (
                    <div className="flex gap-1">
                      {!isSelf && (
                        <Button
                          onClick={() => handleEdit(admin, ctx.refresh)}
                          variant="plain"
                          size="sm"
                          iconOnly
                          disabled={deleting}
                          title={t("Edit")}
                        >
                          <Pencil size={16} />
                        </Button>
                      )}
                      {!isSelf && admin.admin_type === ADMIN_TYPE.ADMIN && (
                        <Button
                          onClick={() => handleDelete(admin, ctx.refresh)}
                          variant="plain"
                          size="sm"
                          iconOnly
                          busy={deleting}
                          disabled={deleting}
                          className="hover:bg-red-50 hover:text-red-600"
                          title={t("Delete")}
                        >
                          <Trash2 size={16} />
                        </Button>
                      )}
                    </div>
                  );
                },
              },
            ]
          : []),
        {
          key: "username",
          label: t("Username"),
          cellClassName: "font-medium",
          render: (admin) => admin.username,
        },
        {
          key: "email",
          label: t("Email"),
          cellClassName: "text-muted",
          render: (admin) => admin.email ?? "—",
        },
        {
          key: "name",
          label: t("Name"),
          render: (admin) => admin.name,
        },
        {
          key: "admin_type",
          label: t("Admin Type"),
          render: (admin) => <TypeBadge type={admin.admin_type} />,
        },
        {
          key: "abilities",
          label: t("Permissions"),
          sortable: false,
          render: (admin) => <PermissionSummary admin={admin} />,
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (admin) => formatDateTime(admin.created_at),
        },
      ]}
      onPostCall={ENABLE_SUMMARY_CARDS ? handleDatatablePostCall : undefined}
      renderTableFooter={({ records }) => {
        const pageDeveloperCount = records.filter(
          (admin) => admin.admin_type === ADMIN_TYPE.DEVELOPER,
        ).length;
        const pageSuperadminCount = records.filter(
          (admin) => admin.admin_type === ADMIN_TYPE.SUPERADMIN,
        ).length;
        const pageAdminCount = records.filter(
          (admin) => admin.admin_type === ADMIN_TYPE.ADMIN,
        ).length;
        return (
          <tr>
            <td colSpan={99} className="px-4 py-2 text-xs text-muted">
              {t("Page rows")}: {records.length}
              {" · "}
              {t("Page developers")}: {pageDeveloperCount}
              {" · "}
              {t("Page super admins")}: {pageSuperadminCount}
              {" · "}
              {t("Page admins")}: {pageAdminCount}
            </td>
          </tr>
        );
      }}
    />
  );
}
