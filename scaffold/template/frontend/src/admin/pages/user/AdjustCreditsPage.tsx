import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import type { UserCreditTransactionDatatableRow } from "@admin/types";
import { ADJUSTABLE_CREDIT_TYPES } from "@admin/types";
import { CREDIT_TYPE_I18N, ADJUSTABLE_CREDIT_TYPE_I18N } from "@admin/constants/enums";
import {
  Button,
  DataTable,
  useAutoForm,
  useModalStore,
  alertSuccess,
  alertError,
  formatDateTime,
} from "@shared/components";
import { api } from "@admin/api";

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

function AdjustCreditForm({
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
    url: "users/credits/adjust",
    method: "post",
    fields: (values) => [
      { name: "username", type: "text", label: t("Username"), placeholder: t("Enter username"), required: true },
      { name: "credit_type", type: "select", label: t("Credit Type"), required: true, placeholder: t("Select"), options: ADJUSTABLE_CREDIT_TYPES.map((value) => ({ value, label: t(ADJUSTABLE_CREDIT_TYPE_I18N[value] ?? value) })) },
      { name: "amount", type: "text", label: t("Amount"), placeholder: "e.g. 100 or -50", required: true },
      { name: "remark", type: "textarea", label: t("Remark"), placeholder: t("Enter remark") },
      { name: "custom_description", type: "checkbox", label: t("Custom Description") },
      ...(values.custom_description === "1"
        ? [{ name: "custom_description_text", type: "text" as const, label: t("Custom Description"), localized: true }]
        : []),
    ],
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Credit adjusted") });
      onCreated();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to adjust credit.")),
      });
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return <form id={formId} onSubmit={submit}>{form}</form>;
}

export default function AdjustCreditsPage() {
  const { t } = useTranslation();
  const refreshRef = useRef<(() => void) | null>(null);

  const handleAdjust = (refresh: () => void) => {
    refreshRef.current = refresh;
    const formId = `credit-adjust-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
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
          {busy ? t("Submitting\u2026") : t("Submit")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Adjust Credit"),
      size: "lg",
      content: (
        <AdjustCreditForm
          onCreated={() => {
            refreshRef.current?.();
          }}
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

  return (
    <DataTable<UserCreditTransactionDatatableRow>
      url="datatable/user_credit_transaction/query"
      title={t("Adjust Credits")}
      subtitle={t("Manage credit adjustments")}
      headerActions={(refresh) => (
        <Button
          onClick={() => handleAdjust(refresh)}
          variant="primary"
          size="sm"
        >
          <Plus size={16} />
          {t("Adjust Credit")}
        </Button>
      )}
      columns={[
        {
          key: "user_username",
          label: t("User"),
          render: (row) => row.user_username ?? row.user_id,
        },
        {
          key: "credit_type",
          label: t("Credit Type"),
          render: (row) => t(CREDIT_TYPE_I18N[row.credit_type] ?? row.credit_type),
        },
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
          key: "admin_username",
          label: t("Adjusted By"),
          cellClassName: "text-muted",
          render: (row) => row.admin_username ?? "\u2014",
        },
        {
          key: "related_key",
          label: t("Related Key"),
          cellClassName: "text-muted",
          render: (row) => row.related_key ?? "\u2014",
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
