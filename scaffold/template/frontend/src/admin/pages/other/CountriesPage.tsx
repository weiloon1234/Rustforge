import { useEffect } from "react";
import { Pencil } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { CountryDatatableRow } from "@admin/types";
import { PERMISSION } from "@admin/types";
import { api } from "@admin/api";
import { useAuthStore } from "@admin/stores/auth";
import { hasPermission } from "@shared/permissions";
import {
  Button,
  DataTable,
  Select,
  alertError,
  alertSuccess,
  formatDateTime,
  useAutoForm,
  useModalStore,
} from "@shared/components";

const COUNTRY_STATUS_ENABLED = "enabled";
const COUNTRY_STATUS_DISABLED = "disabled";
const EMPTY_SCOPES: string[] = [];

function statusBadgeClass(status: string): string {
  if (status === COUNTRY_STATUS_ENABLED) {
    return "bg-emerald-100 text-emerald-700";
  }
  return "bg-gray-100 text-gray-700";
}

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

function EditCountryStatusForm({
  row,
  onUpdated,
  formId,
  onBusyChange,
}: {
  row: CountryDatatableRow;
  onUpdated: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: `countries/${row.iso2}/status`,
    method: "patch",
    fields: [
      {
        name: "status",
        type: "select",
        label: t("Status"),
        options: [
          { value: COUNTRY_STATUS_ENABLED, label: t("Enabled") },
          { value: COUNTRY_STATUS_DISABLED, label: t("Disabled") },
        ],
        required: true,
      },
    ],
    defaults: {
      status: row.status,
    },
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Country status updated") });
      onUpdated();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to update country status.")),
      });
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
      <div className="grid gap-3 sm:grid-cols-2">
        <Select
          containerClassName="mb-0"
          label={t("Country")}
          value={row.iso2}
          disabled
          options={[{ value: row.iso2, label: `${row.name} (${row.iso2})` }]}
        />
        {busy ? (
          <div className="flex items-end text-xs text-muted">{t("Saving…")}</div>
        ) : (
          <div />
        )}
      </div>
      {form}
    </form>
  );
}

export default function CountriesPage() {
  const { t } = useTranslation();
  const scopes = useAuthStore((state) => state.account?.scopes ?? EMPTY_SCOPES);
  const canManage = hasPermission(scopes, PERMISSION.COUNTRY_MANAGE);

  const openEditModal = (row: CountryDatatableRow, refresh: () => void) => {
    const formId = `country-status-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
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
      title: t("Edit Country Status"),
      size: "md",
      content: (
        <EditCountryStatusForm
          row={row}
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

  return (
    <DataTable<CountryDatatableRow>
      url="datatable/country/query"
      title={t("Countries")}
      subtitle={t("Manage country availability")}
      columns={[
        {
          key: "actions",
          label: t("Actions"),
          sortable: false,
          render: (row, ctx) =>
            canManage ? (
              <Button
                type="button"
                onClick={() => openEditModal(row, ctx.refresh)}
                variant="plain"
                size="sm"
                iconOnly
                title={t("Edit")}
              >
                <Pencil size={16} />
              </Button>
            ) : (
              "—"
            ),
        },
        {
          key: "status",
          label: t("Status"),
          render: (row) => (
            <span
              className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${statusBadgeClass(row.status)}`}
            >
              {row.status === COUNTRY_STATUS_ENABLED ? t("Enabled") : t("Disabled")}
            </span>
          ),
        },
        {
          key: "iso2",
          label: t("ISO2"),
          cellClassName: "font-medium",
        },
        {
          key: "iso3",
          label: t("ISO3"),
        },
        {
          key: "name",
          label: t("Name"),
        },
        {
          key: "calling_code",
          label: t("Calling Code"),
          render: (row) => row.calling_code ?? "—",
        },
        {
          key: "region",
          label: t("Region"),
          render: (row) => row.region ?? "—",
        },
        {
          key: "updated_at",
          label: t("Updated At"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => formatDateTime(row.updated_at),
        },
      ]}
    />
  );
}
