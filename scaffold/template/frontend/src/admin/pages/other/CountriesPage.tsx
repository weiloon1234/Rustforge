import { useEffect } from "react";
import { Pencil, Star } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { CountryDatatableRow } from "@admin/types";
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
} from "@shared/components";

const COUNTRY_STATUS_ENABLED = "enabled";
const COUNTRY_STATUS_DISABLED = "disabled";

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

  const { submit, busy, form } = useAutoForm(api, {
    url: `countries/${row.iso2}/status`,
    method: "patch",
    fields: [
      {
        name: "country",
        type: "text",
        label: t("Country"),
        disabled: true,
        span: 1,
      },
      {
        name: "status",
        type: "select",
        label: t("Status"),
        options: [
          { value: COUNTRY_STATUS_ENABLED, label: t("Enabled") },
          { value: COUNTRY_STATUS_DISABLED, label: t("Disabled") },
        ],
        required: true,
        span: 1,
      },
    ],
    defaults: {
      country: `${row.name} (${row.iso2})`,
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
    <form id={formId} onSubmit={submit}>
      {form}
    </form>
  );
}

export default function CountriesPage() {
  const { t } = useTranslation();
  const account = useAuthStore((state) => state.account);
  const canManage = useAuthStore.hasPermission(
    PERMISSION.COUNTRY_MANAGE,
    account,
  );

  const handleSetDefault = async (row: CountryDatatableRow, refresh: () => void) => {
    await alertConfirm({
      title: t("Set Default"),
      message: t("Are you sure you want to set :name as the default country?", {
        name: `${row.name} (${row.iso2})`,
      }),
      confirmText: t("Set Default"),
      callback: async (result) => {
        if (!result.isConfirmed) return;
        try {
          await api.patch(`countries/${row.iso2}/default`);
          alertSuccess({ title: t("Success"), message: t("Country default updated") });
          refresh();
        } catch (error) {
          alertError({
            title: t("Error"),
            message: normalizeErrorMessage(error, t("Failed to set default country.")),
          });
        }
      },
    });
  };

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
              <div className="flex items-center gap-1">
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
                {!row.is_default && (
                  <Button
                    type="button"
                    onClick={() => handleSetDefault(row, ctx.refresh)}
                    variant="plain"
                    size="sm"
                    iconOnly
                    title={t("Set Default")}
                  >
                    <Star size={16} />
                  </Button>
                )}
              </div>
            ) : (
              "—"
            ),
        },
        {
          key: "is_default",
          label: t("Default"),
          render: (row) =>
            row.is_default ? (
              <span className="inline-block rounded-full bg-amber-100 px-2 py-0.5 text-xs font-medium text-amber-700">
                {t("Default")}
              </span>
            ) : null,
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
          label: t("Country Name"),
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
