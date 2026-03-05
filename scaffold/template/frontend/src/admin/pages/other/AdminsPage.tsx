import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Pencil, Trash2 } from "lucide-react";
import type {
  AdminDatatableSummaryOutput,
  AdminDeleteOutput,
  AdminDatatableRow,
  AdminType,
  Permission,
} from "@admin/types";
import { ADMIN_TYPE, PERMISSIONS, PERMISSION_META } from "@admin/types";
import type { ApiResponse } from "@shared/types";
import {
  Button,
  Checkbox,
  DataTable,
  useAutoForm,
  useModalStore,
  alertConfirm,
  alertSuccess,
  alertError,
  formatDateTime,
} from "@shared/components";
import type { DataTablePostCallEvent } from "@shared/components";
import { api } from "@admin/api";

const TYPE_COLORS: Record<AdminType, string> = {
  [ADMIN_TYPE.DEVELOPER]: "bg-purple-100 text-purple-700",
  [ADMIN_TYPE.SUPERADMIN]: "bg-amber-100 text-amber-700",
  [ADMIN_TYPE.ADMIN]: "bg-blue-100 text-blue-700",
};

const ADMIN_PERMISSION_META = PERMISSION_META.filter(
  (meta) => meta.guard.toLowerCase() === "admin",
);

const ENABLE_SUMMARY_CARDS = true;

function TypeBadge({ type }: { type: AdminType }) {
  return (
    <span
      className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${TYPE_COLORS[type] ?? "bg-gray-100 text-gray-700"}`}
    >
      {type}
    </span>
  );
}

function PermissionSummary({ admin }: { admin: AdminDatatableRow }) {
  const { t } = useTranslation();
  if (
    admin.admin_type !== ADMIN_TYPE.ADMIN ||
    admin.abilities.includes("*")
  ) {
    return (
      <span className="inline-block rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-700">
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
                return (
                  <div
                    key={ability}
                    className="rounded-lg border border-border bg-surface px-3 py-2"
                  >
                    <p className="text-sm font-medium">{t(meta?.label ?? ability)}</p>
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
    <Button
      type="button"
      variant="plain"
      size="sm"
      className="rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-700 hover:bg-gray-200"
      onClick={openPermissionModal}
    >
      {t(":count Permissions", { count: abilities.length })}
    </Button>
  );
}

function PermissionCheckboxes({
  abilities,
  onChange,
}: {
  abilities: Permission[];
  onChange: (next: Permission[]) => void;
}) {
  const { t } = useTranslation();
  return (
    <fieldset className="space-y-2">
      <legend className="text-sm font-medium">{t("Permissions")}</legend>
      <div className="flex flex-wrap gap-x-6 gap-y-1">
        {ADMIN_PERMISSION_META.map((meta) => (
          <Checkbox
            key={meta.key}
            label={t(meta.label)}
            checked={abilities.includes(meta.key as Permission)}
            onChange={(e) => {
              const permission = meta.key as Permission;
              if (e.target.checked) {
                onChange([...abilities, permission]);
              } else {
                onChange(abilities.filter((value) => value !== permission));
              }
            }}
          />
        ))}
      </div>
    </fieldset>
  );
}

function CreateAdminForm({
  onCreated,
  formId,
  onBusyChange,
}: {
  onCreated: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);
  const [abilities, setAbilities] = useState<Permission[]>([]);

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "admins",
    method: "post",
    extraPayload: { abilities },
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
    ],
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Admin created") });
      onCreated();
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit} className="space-y-4">
      {errors.general && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {errors.general}
        </p>
      )}
      {form}
      <PermissionCheckboxes abilities={abilities} onChange={setAbilities} />
    </form>
  );
}

function EditAdminForm({
  admin,
  onUpdated,
  formId,
  onBusyChange,
}: {
  admin: AdminDatatableRow;
  onUpdated: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);
  const isNormalAdmin = admin.admin_type === ADMIN_TYPE.ADMIN;
  const [abilities, setAbilities] = useState<Permission[]>(
    admin.abilities.filter((value): value is Permission =>
      PERMISSIONS.includes(value as Permission),
    ),
  );

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: `admins/${admin.id}`,
    method: "patch",
    extraPayload: isNormalAdmin ? { abilities } : {},
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

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit} className="space-y-4">
      {errors.general && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {errors.general}
        </p>
      )}
      {form}
      {isNormalAdmin && (
        <PermissionCheckboxes abilities={abilities} onChange={setAbilities} />
      )}
    </form>
  );
}

export default function AdminsPage() {
  const { t } = useTranslation();
  const [summary, setSummary] = useState<AdminDatatableSummaryOutput | null>(
    null,
  );
  const [deletingAdminId, setDeletingAdminId] = useState<string | null>(null);
  const summaryRequestId = useRef(0);

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
          onBusyChange={(busy) => {
            if (!modalId) return;
            useModalStore.getState().update(modalId, { footer: renderFooter(busy) });
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
          onBusyChange={(busy) => {
            if (!modalId) return;
            useModalStore.getState().update(modalId, { footer: renderFooter(busy) });
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
      headerActions={(refresh) => (
        <Button
          onClick={() => handleCreate(refresh)}
          variant="primary"
          size="sm"
        >
          <Plus size={16} />
          {t("Create Admin")}
        </Button>
      )}
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
        {
          key: "actions",
          label: t("Actions"),
          sortable: false,
          render: (admin, ctx) => {
            const deleting = deletingAdminId === admin.id;
            return (
              <div className="flex gap-1">
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
                {admin.admin_type === ADMIN_TYPE.ADMIN && (
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
