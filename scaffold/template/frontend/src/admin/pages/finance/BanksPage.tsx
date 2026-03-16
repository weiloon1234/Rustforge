import { useMemo, useEffect, useRef } from "react";
import { Pencil, Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { BankDatatableRow } from "@admin/types";
import { PERMISSION } from "@admin/types";
import { api } from "@admin/api";
import { useAuthStore } from "@admin/stores/auth";
import { availableCountries } from "@shared/countryRuntime";
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

function BankForm({
  bankId,
  defaults,
  onSaved,
  formId,
  onBusyChange,
}: {
  bankId?: string;
  defaults?: Record<string, unknown>;
  onSaved: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  const countryOptions = useMemo(
    () => availableCountries().map((c) => ({ value: c.iso2, label: `${c.flag_emoji ?? ""} ${c.name} (${c.iso2})`.trim() })),
    [],
  );

  const { submit, busy, form } = useAutoForm(api, {
    url: bankId ? `banks/${bankId}` : "banks",
    method: bankId ? "put" : "post",
    bodyType: "multipart",
    fields: [
      { name: "country_iso2", type: "select", label: t("Country"), required: true, options: countryOptions, placeholder: t("Select country") },
      { name: "name", type: "text", label: t("Name"), required: true },
      { name: "code", type: "text", label: t("Code") },
      { name: "logo", type: "file", label: t("Logo"), accept: "image/*" },
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
      alertSuccess({ title: t("Success"), message: bankId ? t("Bank updated") : t("Bank created") });
      onSaved();
    },
    onError: (error) => {
      alertError({ title: t("Error"), message: normalizeErrorMessage(error, t("Failed to save bank.")) });
    },
  });

  useEffect(() => { onBusyChange(busy); }, [busy, onBusyChange]);

  return <form id={formId} onSubmit={submit}>{form}</form>;
}

export default function BanksPage() {
  const { t } = useTranslation();
  const refreshRef = useRef<(() => void) | null>(null);
  const account = useAuthStore((s) => s.account);
  const canManage = useAuthStore.hasPermission(PERMISSION.BANK_MANAGE, account);

  const openFormModal = (row: BankDatatableRow | null, refresh: () => void) => {
    refreshRef.current = refresh;
    const isEdit = !!row;
    const formId = `bank-form-${Date.now()}`;
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
      title: isEdit ? t("Edit Bank") : t("Create Bank"),
      size: "lg",
      content: (
        <BankForm
          bankId={row?.id}
          defaults={row ? {
            country_iso2: row.country_iso2,
            name: row.name,
            code: row.code ?? "",
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

  const handleDelete = async (row: BankDatatableRow, refresh: () => void) => {
    await alertConfirm({
      title: t("Delete Bank"),
      message: t("Are you sure you want to delete bank :name?", { name: row.name }),
      confirmText: t("Delete"),
      callback: async (result) => {
        if (!result.isConfirmed) return;
        try {
          await api.delete(`banks/${row.id}`);
          alertSuccess({ title: t("Success"), message: t("Bank deleted") });
          refresh();
        } catch (error) {
          alertError({ title: t("Error"), message: normalizeErrorMessage(error, t("Failed to delete bank.")) });
        }
      },
    });
  };

  return (
    <DataTable<BankDatatableRow>
      url="datatable/bank/query"
      title={t("Banks")}
      subtitle={t("Manage bank list")}
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
              key: "actions" as keyof BankDatatableRow,
              label: t("Actions"),
              sortable: false,
              render: (row: BankDatatableRow, ctx: DataTableCellContext<BankDatatableRow>) => (
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
        { key: "country_iso2", label: t("Country") },
        { key: "name", label: t("Name"), cellClassName: "font-medium" },
        { key: "code", label: t("Code"), render: (row: BankDatatableRow) => row.code ?? "\u2014" },
        {
          key: "status",
          label: t("Status"),
          render: (row: BankDatatableRow) => (
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
          render: (row: BankDatatableRow) => formatDateTime(row.updated_at),
        },
      ]}
    />
  );
}
