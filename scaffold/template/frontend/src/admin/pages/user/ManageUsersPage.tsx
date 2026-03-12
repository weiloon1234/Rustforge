import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Pencil, ShieldBan, ShieldCheck } from "lucide-react";
import type {
  AdminMeOutput,
  UserDatatableRow,
  UserDatatableSummaryOutput,
  UserCreditTransactionDatatableRow,
  UserBanStatus,
  BatchResolveOutput,
  CreditType,
} from "@admin/types";
import { PERMISSION, USER_BAN_STATUS, CREDIT_TYPE } from "@admin/types";
import { CREDIT_TYPE_I18N, BAN_STATUS_I18N } from "@admin/constants/enums";
import type { ApiResponse } from "@shared/types";
import {
  Button,
  DataTable,
  type DataTableCellContext,
  TextInput,
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

const BAN_COLORS: Record<UserBanStatus, { className: string; labelKey: string }> = {
  "0": { className: "bg-emerald-100 text-emerald-700", labelKey: BAN_STATUS_I18N["0"] },
  "1": { className: "bg-red-100 text-red-700", labelKey: BAN_STATUS_I18N["1"] },
};

function BanBadge({ status }: { status: UserBanStatus }) {
  const { t } = useTranslation();
  const config = BAN_COLORS[status] ?? BAN_COLORS[USER_BAN_STATUS._0];
  return (
    <span
      className={`inline-flex min-h-6 items-center rounded-full px-2 py-0.5 text-xs font-medium ${config.className}`}
    >
      {t(config.labelKey)}
    </span>
  );
}

const ENABLE_SUMMARY_CARDS = true;


function CreditTransactionsModal({
  userId,
  creditType,
}: {
  userId?: string;
  creditType: CreditType;
}) {
  const { t } = useTranslation();

  const extraBody: Record<string, string> = { "f-credit_type": creditType };
  if (userId) {
    extraBody["f-user_id"] = userId;
  }

  return (
    <DataTable<UserCreditTransactionDatatableRow>
      url="datatable/user_credit_transaction/query"
      extraBody={extraBody}
      perPage={10}
      showRefresh={false}
      enableAutoRefresh={false}
      columns={[
        ...(!userId
          ? [
              {
                key: "user_username" as const,
                label: t("User"),
                render: (row: UserCreditTransactionDatatableRow) =>
                  row.user_username ?? row.user_id,
              },
            ]
          : []),
        {
          key: "amount",
          label: t("Amount"),
          cellClassName: "tabular-nums",
          render: (row) => {
            const num = parseFloat(row.amount);
            const color =
              num > 0
                ? "text-emerald-600"
                : num < 0
                  ? "text-red-600"
                  : "";
            return <span className={color}>{row.amount}</span>;
          },
        },
        {
          key: "transaction_type_explained",
          label: t("Description"),
          render: (row) => row.transaction_type_explained,
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => formatDateTime(row.created_at),
        },
      ]}
    />
  );
}

function CreateUserForm({
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

  const { submit, busy, form } = useAutoForm(api, {
    url: "users",
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
      },
      {
        name: "email",
        type: "email",
        label: t("Email"),
        placeholder: t("Enter email"),
      },
      {
        name: "password",
        type: "password",
        label: t("Password"),
        placeholder: t("Enter password"),
        required: true,
      },
      {
        name: "contact",
        type: "contact",
      },
      {
        name: "introducer_username",
        type: "text",
        label: t("Introducer Username"),
        placeholder: t("Enter introducer username"),
      },
    ],
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("User created") });
      onCreated();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to create user.")),
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

function EditUserForm({
  user,
  introducerUsername,
  onUpdated,
  formId,
  onBusyChange,
}: {
  user: UserDatatableRow;
  introducerUsername: string;
  onUpdated: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  const { submit, busy, form } = useAutoForm(api, {
    url: `users/${user.id}`,
    method: "patch",
    extraPayload: { id: user.id },
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
      },
      {
        name: "email",
        type: "email",
        label: t("Email"),
        placeholder: t("Enter email"),
      },
      {
        name: "password",
        type: "password",
        label: t("Password"),
        placeholder: t("Enter password"),
        notes: t("Leave blank to keep current password"),
      },
      {
        name: "contact",
        type: "contact",
      },
    ],
    defaults: {
      username: user.username,
      name: user.name ?? "",
      email: user.email ?? "",
      country_iso2: user.country_iso2 ?? "",
      contact_number: user.contact_number ?? "",
    },
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("User updated") });
      onUpdated();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to update user.")),
      });
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit}>
      <div className="rf-field">
        <TextInput
          label={t("Introducer")}
          value={introducerUsername}
          disabled
        />
      </div>
      <div className="rf-field">
        <TextInput label={t("Credit 1")} value={user.credit_1 ?? "0"} disabled />
      </div>
      <div className="rf-field">
        <TextInput label={t("Credit 2")} value={user.credit_2 ?? "0"} disabled />
      </div>
      {form}
    </form>
  );
}

function canManageUsers(account: AdminMeOutput | null): boolean {
  if (!account) return false;
  return useAuthStore.hasPermission(PERMISSION.USER_MANAGE, account);
}

export default function ManageUsersPage() {
  const { t } = useTranslation();
  const account = useAuthStore((state) => state.account);
  const [summary, setSummary] = useState<UserDatatableSummaryOutput | null>(null);
  const [banningUserId, setBanningUserId] = useState<string | null>(null);
  const [introducerMap, setIntroducerMap] = useState<Map<string, { username: string; name: string | null }>>(new Map());
  const summaryRequestId = useRef(0);
  const canManage = useMemo(() => canManageUsers(account), [account]);

  const handleCreditClick = (user: UserDatatableRow, creditType: CreditType) => {
    const creditLabel = t(CREDIT_TYPE_I18N[creditType] ?? creditType);
    useModalStore.getState().open({
      title: `${user.username} — ${creditLabel}`,
      size: "xl",
      content: (
        <CreditTransactionsModal
          userId={user.id}
          creditType={creditType}
        />
      ),
      footer: (
        <Button
          type="button"
          variant="secondary"
          onClick={() => useModalStore.getState().close()}
        >
          {t("Close")}
        </Button>
      ),
    });
  };

  const handleSummaryCardCreditClick = (creditType: CreditType) => {
    const creditLabel = t(CREDIT_TYPE_I18N[creditType] ?? creditType);
    useModalStore.getState().open({
      title: creditLabel,
      size: "xl",
      content: (
        <CreditTransactionsModal creditType={creditType} />
      ),
      footer: (
        <Button
          type="button"
          variant="secondary"
          onClick={() => useModalStore.getState().close()}
        >
          {t("Close")}
        </Button>
      ),
    });
  };

  const handleDatatablePostCall = (
    event: DataTablePostCallEvent<UserDatatableRow>,
  ) => {
    // Batch resolve introducer usernames
    if (event.response && !event.error) {
      const rows = event.response?.records ?? [];
      const introducerIds = [
        ...new Set(
          rows
            .map((row: UserDatatableRow) => row.introducer_user_id)
            .filter((id: string | null): id is string => id != null),
        ),
      ];
      if (introducerIds.length > 0) {
        void api
          .post<ApiResponse<BatchResolveOutput>>("users/batch_resolve", {
            ids: introducerIds,
          })
          .then((res) => {
            const entries = res.data?.data?.entries ?? [];
            setIntroducerMap((prev) => {
              const next = new Map(prev);
              for (const entry of entries) {
                next.set(String(entry.id), { username: entry.username, name: entry.name });
              }
              return next;
            });
          })
          .catch(() => {
            // silently ignore resolve errors
          });
      }
    }

    // Summary cards
    if (!ENABLE_SUMMARY_CARDS) return;
    if (!event.response || event.error) {
      setSummary(null);
      return;
    }

    const requestId = ++summaryRequestId.current;
    const payload: Record<string, unknown> = {
      base: { include_meta: false },
      ...event.filters.applied,
    };

    void api
      .post<ApiResponse<UserDatatableSummaryOutput>>(
        "datatable/user/summary",
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
    const formId = `user-create-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
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
          {busy ? t("Creating\u2026") : t("Create")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Create User"),
      size: "lg",
      content: (
        <CreateUserForm
          onCreated={refresh}
          formId={formId}
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

  const handleEdit = (user: UserDatatableRow, refresh: () => void) => {
    const formId = `user-edit-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    let modalId = "";

    const introducerUsername = user.introducer_user_id
      ? (introducerMap.get(user.introducer_user_id)?.username ?? "\u2014")
      : "\u2014";

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
          {busy ? t("Saving\u2026") : t("Save")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Edit User"),
      size: "lg",
      content: (
        <EditUserForm
          user={user}
          introducerUsername={introducerUsername}
          onUpdated={refresh}
          formId={formId}
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

  const handleBan = async (
    user: UserDatatableRow,
    refresh: () => void,
  ) => {
    const isBanned = user.ban === USER_BAN_STATUS._1;
    const newStatus: UserBanStatus = isBanned ? USER_BAN_STATUS._0 : USER_BAN_STATUS._1;
    const confirmTitle = isBanned ? t("Unban") : t("Ban");
    const confirmMessage = isBanned
      ? t('Are you sure you want to unban ":username"?', { username: user.username })
      : t('Are you sure you want to ban ":username"?', { username: user.username });

    await alertConfirm({
      title: confirmTitle,
      message: confirmMessage,
      confirmText: confirmTitle,
      callback: async (result) => {
        if (result.isConfirmed) {
          if (banningUserId === user.id) return;
          setBanningUserId(user.id);
          try {
            await api.patch<ApiResponse<{ banned: boolean }>>(
              `users/${user.id}/ban`,
              { ban: newStatus },
            );
            alertSuccess({
              title: t("Success"),
              message: isBanned ? t("User unbanned") : t("User banned"),
            });
            refresh();
          } catch (error) {
            alertError({
              title: t("Error"),
              message: normalizeErrorMessage(error, t("Failed to update ban status.")),
            });
          } finally {
            setBanningUserId(null);
          }
        }
      },
    });
  };

  return (
    <DataTable<UserDatatableRow>
      url="datatable/user/query"
      title={t("Users")}
      subtitle={t("Manage user accounts")}
      headerActions={
        canManage
          ? (refresh) => (
              <Button
                onClick={() => handleCreate(refresh)}
                variant="primary"
                size="sm"
              >
                <Plus size={16} />
                {t("Create User")}
              </Button>
            )
          : undefined
      }
      headerContent={
        ENABLE_SUMMARY_CARDS && summary ? (
          <div className="grid gap-2 sm:grid-cols-5">
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Filtered Total")}</p>
              <p className="font-semibold">
                {summary.total_user_count ?? summary.total_filtered}
              </p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Filtered")}</p>
              <p className="font-semibold">{summary.total_filtered}</p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Banned")}</p>
              <p className="font-semibold">{summary.banned_count}</p>
            </div>
            <button
              type="button"
              className="cursor-pointer rounded-lg border border-border bg-surface px-3 py-2 text-left text-sm transition-colors hover:bg-muted/50"
              onClick={() => handleSummaryCardCreditClick(CREDIT_TYPE._1)}
            >
              <p className="text-xs text-muted">{t("Total Credit 1")}</p>
              <p className="font-semibold tabular-nums">{summary.total_credit_1}</p>
            </button>
            <button
              type="button"
              className="cursor-pointer rounded-lg border border-border bg-surface px-3 py-2 text-left text-sm transition-colors hover:bg-muted/50"
              onClick={() => handleSummaryCardCreditClick(CREDIT_TYPE._2)}
            >
              <p className="text-xs text-muted">{t("Total Credit 2")}</p>
              <p className="font-semibold tabular-nums">{summary.total_credit_2}</p>
            </button>
          </div>
        ) : undefined
      }
      columns={[
        ...(canManage
          ? [
              {
                key: "actions",
                label: t("Actions"),
                sortable: false,
                render: (
                  user: UserDatatableRow,
                  ctx: DataTableCellContext<UserDatatableRow>,
                ) => {
                  const isBanned = user.ban === USER_BAN_STATUS._1;
                  const banning = banningUserId === user.id;
                  return (
                    <div className="flex gap-1">
                      <Button
                        onClick={() => handleEdit(user, ctx.refresh)}
                        variant="plain"
                        size="sm"
                        iconOnly
                        disabled={banning}
                        title={t("Edit")}
                      >
                        <Pencil size={16} />
                      </Button>
                      <Button
                        onClick={() => handleBan(user, ctx.refresh)}
                        variant="plain"
                        size="sm"
                        iconOnly
                        busy={banning}
                        disabled={banning}
                        className={
                          isBanned
                            ? "hover:bg-emerald-50 hover:text-emerald-600"
                            : "hover:bg-red-50 hover:text-red-600"
                        }
                        title={isBanned ? t("Unban") : t("Ban")}
                      >
                        {isBanned ? <ShieldCheck size={16} /> : <ShieldBan size={16} />}
                      </Button>
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
          render: (user) => user.username,
        },
        {
          key: "email",
          label: t("Email"),
          cellClassName: "text-muted",
          render: (user) => user.email ?? "\u2014",
        },
        {
          key: "name",
          label: t("Name"),
          render: (user) => user.name ?? "\u2014",
        },
        {
          key: "ban",
          label: t("Ban Status"),
          render: (user) => <BanBadge status={user.ban} />,
        },
        {
          key: "country_iso2",
          label: t("Country ISO2"),
          cellClassName: "text-muted",
          render: (user) => user.country_iso2 ?? "\u2014",
        },
        {
          key: "credit_1",
          label: t("Credit 1"),
          cellClassName: "tabular-nums",
          render: (user) => (
            <button
              type="button"
              className="cursor-pointer text-blue-500 hover:text-blue-700 hover:underline"
              onClick={() => handleCreditClick(user, CREDIT_TYPE._1)}
            >
              {user.credit_1 ?? "0"}
            </button>
          ),
        },
        {
          key: "credit_2",
          label: t("Credit 2"),
          cellClassName: "tabular-nums",
          render: (user) => (
            <button
              type="button"
              className="cursor-pointer text-blue-500 hover:text-blue-700 hover:underline"
              onClick={() => handleCreditClick(user, CREDIT_TYPE._2)}
            >
              {user.credit_2 ?? "0"}
            </button>
          ),
        },
        {
          key: "introducer_user_id",
          label: t("Introducer"),
          sortable: false,
          render: (user) => {
            if (!user.introducer_user_id) return "\u2014";
            const resolved = introducerMap.get(user.introducer_user_id);
            return resolved?.username ?? user.introducer_user_id;
          },
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (user) => formatDateTime(user.created_at),
        },
      ]}
      onPostCall={handleDatatablePostCall}
      renderTableFooter={({ records }) => {
        const bannedCount = records.filter(
          (user) => user.ban === USER_BAN_STATUS._1,
        ).length;
        return (
          <tr>
            <td colSpan={99} className="px-4 py-2 text-xs text-muted">
              {t("Page rows")}: {records.length}
              {" \u00B7 "}
              {t("Banned")}: {bannedCount}
            </td>
          </tr>
        );
      }}
    />
  );
}
