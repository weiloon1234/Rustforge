import { useEffect, useRef } from "react";
import { Pencil, Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { CompanyBankAccountDatatableRow } from "@admin/types";
import { PERMISSION } from "@admin/types";
import { api } from "@admin/api";
import { useAuthStore } from "@admin/stores/auth";
import {
  Button,
  DataTable,
  alertConfirm,
  alertError,
  alertSuccess,
  formatDateTime,
  useAutoForm,
  useModalStore,
  type AutoFormDefaultValue,
} from "@shared/components";
import type { DataTableCellContext } from "@shared/components/DataTable";

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

const STATUS_COLORS: Record<string, string> = {
  "1": "bg-emerald-100 text-emerald-700",
  "2": "bg-gray-100 text-gray-700",
};

const STATUS_LABELS: Record<string, string> = {
  "1": "Enabled",
  "2": "Disabled",
};

function CompanyBankAccountForm({
  accountId,
  defaults,
  onSaved,
  formId,
  onBusyChange,
}: {
  accountId?: string;
  defaults?: Record<string, unknown>;
  onSaved: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  const { submit, busy, form } = useAutoForm(api, {
    url: accountId ? `company_bank_accounts/${accountId}` : "company_bank_accounts",
    method: accountId ? "put" : "post",
    fields: [
      { name: "bank_id", type: "text", label: t("Bank ID"), required: true },
      { name: "account_name", type: "text", label: t("Account Name"), required: true },
      { name: "account_number", type: "text", label: t("Account Number"), required: true },
      {
        name: "status",
        type: "select",
        label: t("Status"),
        required: true,
        options: [
          { value: "1", label: t("Enabled") },
          { value: "2", label: t("Disabled") },
        ],
      },
      { name: "sort_order", type: "number", label: t("Sort Order") },
    ],
    defaults: (defaults ?? { status: "1", sort_order: 0 }) as Record<string, AutoFormDefaultValue>,
    onSuccess: () => {
      close();
      alertSuccess({
        title: t("Success"),
        message: accountId ? t("Company bank account updated") : t("Company bank account created"),
      });
      onSaved();
    },
    onError: (error) => {
      alertError({ title: t("Error"), message: normalizeErrorMessage(error, t("Failed to save company bank account.")) });
    },
  });

  useEffect(() => { onBusyChange(busy); }, [busy, onBusyChange]);

  return <form id={formId} onSubmit={submit}>{form}</form>;
}

export default function CompanyBankAccountsPage() {
  const { t } = useTranslation();
  const refreshRef = useRef<(() => void) | null>(null);
  const account = useAuthStore((s) => s.account);
  const canManage = useAuthStore.hasPermission(PERMISSION.COMPANY_BANK_ACCOUNT_MANAGE, account);

  const openFormModal = (row: CompanyBankAccountDatatableRow | null, refresh: () => void) => {
    refreshRef.current = refresh;
    const isEdit = !!row;
    const formId = `cba-form-${Date.now()}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button type="button" onClick={() => useModalStore.getState().close()} variant="secondary" disabled={busy}>
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Saving\u2026") : t("Save")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: isEdit ? t("Edit Company Bank Account") : t("Create Company Bank Account"),
      size: "lg",
      content: (
        <CompanyBankAccountForm
          accountId={row?.id}
          defaults={row ? {
            bank_id: row.bank_id,
            account_name: row.account_name,
            account_number: row.account_number,
            status: String(row.status),
            sort_order: row.sort_order,
          } : undefined}
          onSaved={() => refreshRef.current?.()}
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

  const handleDelete = async (row: CompanyBankAccountDatatableRow, refresh: () => void) => {
    await alertConfirm({
      title: t("Delete Company Bank Account"),
      message: t("Are you sure you want to delete account :name?", { name: row.account_name }),
      confirmText: t("Delete"),
      callback: async (result) => {
        if (!result.isConfirmed) return;
        try {
          await api.delete(`company_bank_accounts/${row.id}`);
          alertSuccess({ title: t("Success"), message: t("Company bank account deleted") });
          refresh();
        } catch (error) {
          alertError({ title: t("Error"), message: normalizeErrorMessage(error, t("Failed to delete company bank account.")) });
        }
      },
    });
  };

  return (
    <DataTable<CompanyBankAccountDatatableRow>
      url="datatable/company_bank_account/query"
      title={t("Company Bank Accounts")}
      subtitle={t("Manage company bank accounts for fiat deposits")}
      headerActions={
        canManage ? (
          <Button size="sm" variant="primary" onClick={() => openFormModal(null, () => refreshRef.current?.())}>
            <Plus size={16} className="mr-1" /> {t("Create")}
          </Button>
        ) : undefined
      }
      columns={[
        ...(canManage
          ? [{
              key: "actions" as keyof CompanyBankAccountDatatableRow,
              label: t("Actions"),
              sortable: false,
              render: (row: CompanyBankAccountDatatableRow, ctx: DataTableCellContext<CompanyBankAccountDatatableRow>) => (
                <div className="flex items-center gap-1">
                  <Button type="button" onClick={() => openFormModal(row, ctx.refresh)} variant="plain" size="sm" iconOnly title={t("Edit")}>
                    <Pencil size={16} />
                  </Button>
                  <Button type="button" onClick={() => handleDelete(row, ctx.refresh)} variant="plain" size="sm" iconOnly title={t("Delete")}>
                    <Trash2 size={16} />
                  </Button>
                </div>
              ),
            }]
          : []),
        { key: "id", label: t("ID"), cellClassName: "tabular-nums text-muted" },
        { key: "bank_name", label: t("Bank"), render: (row: CompanyBankAccountDatatableRow) => row.bank_name ?? row.bank_id },
        { key: "account_name", label: t("Account Name"), cellClassName: "font-medium" },
        { key: "account_number", label: t("Account Number") },
        {
          key: "status",
          label: t("Status"),
          render: (row: CompanyBankAccountDatatableRow) => (
            <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${STATUS_COLORS[row.status] ?? "bg-gray-100 text-gray-800"}`}>
              {row.status_label || t(STATUS_LABELS[row.status] ?? "Unknown")}
            </span>
          ),
        },
        { key: "sort_order", label: t("Sort"), cellClassName: "tabular-nums" },
        {
          key: "updated_at",
          label: t("Updated At"),
          cellClassName: "tabular-nums text-muted",
          render: (row: CompanyBankAccountDatatableRow) => formatDateTime(row.updated_at),
        },
      ]}
    />
  );
}
