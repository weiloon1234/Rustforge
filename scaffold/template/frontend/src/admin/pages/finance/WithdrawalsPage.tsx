import { useRef } from "react";
import { useTranslation } from "react-i18next";
import type { WithdrawalDatatableRow } from "@admin/types";
import { CREDIT_TYPE_I18N, WITHDRAWAL_METHOD_I18N } from "@admin/constants/enums";
import { PERMISSION } from "@admin/types";
import { useAuthStore } from "@admin/stores/auth";
import {
  Button,
  DataTable,
  useAutoForm,
  useModalStore,
  alertSuccess,
  alertError,
  formatDateTime,
  moneyFormat,
} from "@shared/components";
import type { DataTableCellContext } from "@shared/components/DataTable";
import { api } from "@admin/api";

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

const STATUS_COLORS: Record<string, string> = {
  "1": "bg-yellow-100 text-yellow-800", // Pending
  "2": "bg-blue-100 text-blue-800",     // Processing
  "3": "bg-green-100 text-green-800",   // Approved
  "4": "bg-red-100 text-red-800",       // Rejected
};

const STATUS_LABELS: Record<string, string> = {
  "1": "Pending",
  "2": "Processing",
  "3": "Approved",
  "4": "Rejected",
};

function ReviewWithdrawalForm({
  withdrawalId,
  currentStatus,
  onReviewed,
  formId,
  onBusyChange,
}: {
  withdrawalId: string;
  currentStatus: string;
  onReviewed: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  // Build available actions based on current status
  const actionOptions = [];
  if (currentStatus === "1") {
    // Pending: can Process or Reject
    actionOptions.push({ value: "1", label: t("Process") });
    actionOptions.push({ value: "3", label: t("Reject") });
  } else if (currentStatus === "2") {
    // Processing: can Approve or Reject
    actionOptions.push({ value: "2", label: t("Approve") });
    actionOptions.push({ value: "3", label: t("Reject") });
  }

  const { submit, busy, form } = useAutoForm(api, {
    url: `withdrawals/${withdrawalId}/review`,
    method: "post",
    fields: [
      {
        name: "action",
        type: "select",
        label: t("Action"),
        required: true,
        placeholder: t("Select action"),
        options: actionOptions,
      },
      {
        name: "admin_remark",
        type: "textarea",
        label: t("Admin Remark"),
        placeholder: t("Enter remark (optional)"),
      },
    ],
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Withdrawal reviewed") });
      onReviewed();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to review withdrawal.")),
      });
    },
  });

  const prevBusy = useRef(false);
  if (prevBusy.current !== busy) {
    prevBusy.current = busy;
    onBusyChange(busy);
  }

  return <form id={formId} onSubmit={submit}>{form}</form>;
}

function UploadReceiptForm({
  entityId,
  onUploaded,
  formId,
  onBusyChange,
}: {
  entityId: string;
  onUploaded: () => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  const { submit, busy, form } = useAutoForm(api, {
    url: `withdrawals/${entityId}/upload-receipt`,
    method: "post",
    bodyType: "multipart",
    fields: [
      {
        name: "receipt",
        type: "file",
        label: t("Receipt Image"),
        required: true,
      },
    ],
    onSuccess: () => {
      close();
      alertSuccess({ title: t("Success"), message: t("Receipt uploaded") });
      onUploaded();
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to upload receipt.")),
      });
    },
  });

  const prevBusy = useRef(false);
  if (prevBusy.current !== busy) {
    prevBusy.current = busy;
    onBusyChange(busy);
  }

  return <form id={formId} onSubmit={submit}>{form}</form>;
}

export default function WithdrawalsPage() {
  const { t } = useTranslation();
  const refreshRef = useRef<(() => void) | null>(null);
  const account = useAuthStore((s) => s.account);
  const canManage = useAuthStore.hasPermission(PERMISSION.WITHDRAWAL_MANAGE, account);

  const openReviewModal = (row: WithdrawalDatatableRow, refresh: () => void) => {
    refreshRef.current = refresh;
    const formId = `withdrawal-review-${Date.now()}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button type="button" onClick={() => useModalStore.getState().close()} variant="secondary" disabled={busy}>
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Submitting\u2026") : t("Submit")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Review Withdrawal #{{id}}", { id: row.id }),
      size: "lg",
      content: (
        <div>
          <div className="mb-4 grid grid-cols-2 gap-2 text-sm">
            <div><span className="text-muted">{t("Amount")}:</span> {moneyFormat(parseFloat(row.amount))}</div>
            <div><span className="text-muted">{t("Fee")}:</span> {moneyFormat(parseFloat(row.fee))}</div>
            <div><span className="text-muted">{t("Net Amount")}:</span> {moneyFormat(parseFloat(row.net_amount))}</div>
            <div><span className="text-muted">{t("Credit Type")}:</span> {t(CREDIT_TYPE_I18N[row.credit_type] ?? row.credit_type)}</div>
          </div>
          <ReviewWithdrawalForm
            withdrawalId={row.id}
            currentStatus={row.status}
            onReviewed={() => refreshRef.current?.()}
            formId={formId}
            onBusyChange={(busy) => {
              if (!modalId) return;
              useModalStore.getState().update(modalId, { footer: renderFooter(busy) });
            }}
          />
        </div>
      ),
      footer: renderFooter(false),
    });
  };

  const openUploadReceiptModal = (row: WithdrawalDatatableRow, refresh: () => void) => {
    refreshRef.current = refresh;
    const formId = `withdrawal-receipt-${Date.now()}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button type="button" onClick={() => useModalStore.getState().close()} variant="secondary" disabled={busy}>
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Uploading\u2026") : t("Upload")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Upload Receipt for Withdrawal #{{id}}", { id: row.id }),
      size: "md",
      content: (
        <UploadReceiptForm
          entityId={row.id}
          onUploaded={() => refreshRef.current?.()}
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
    <DataTable<WithdrawalDatatableRow>
      url="datatable/withdrawal/query"
      title={t("Withdrawals")}
      subtitle={t("Manage withdrawal requests")}
      columns={[
        {
          key: "id",
          label: t("ID"),
          cellClassName: "tabular-nums text-muted",
        },
        {
          key: "owner_id",
          label: t("Owner"),
          render: (row) => row.owner_name ?? row.owner_id,
        },
        {
          key: "credit_type",
          label: t("Credit Type"),
          render: (row) => t(CREDIT_TYPE_I18N[row.credit_type] ?? row.credit_type),
        },
        {
          key: "withdrawal_method",
          label: t("Method"),
          render: (row) => t(WITHDRAWAL_METHOD_I18N[row.withdrawal_method] ?? row.withdrawal_method),
        },
        {
          key: "bank_name",
          label: t("Destination"),
          render: (row) => row.bank_name ?? row.crypto_network_name ?? "\u2014",
        },
        {
          key: "amount",
          label: t("Amount"),
          cellClassName: "tabular-nums",
          render: (row) => moneyFormat(parseFloat(row.amount)),
        },
        {
          key: "fee",
          label: t("Fee"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => moneyFormat(parseFloat(row.fee)),
        },
        {
          key: "net_amount",
          label: t("Net"),
          cellClassName: "tabular-nums",
          render: (row) => moneyFormat(parseFloat(row.net_amount)),
        },
        {
          key: "status",
          label: t("Status"),
          render: (row) => (
            <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${STATUS_COLORS[row.status] ?? "bg-gray-100 text-gray-800"}`}>
              {row.status_label || t(STATUS_LABELS[row.status] ?? "Unknown")}
            </span>
          ),
        },
        {
          key: "admin_username",
          label: t("Reviewed By"),
          cellClassName: "text-muted",
          render: (row) => row.admin_username ?? "\u2014",
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => formatDateTime(row.created_at),
        },
        {
          key: "reviewed_at",
          label: t("Reviewed At"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => row.reviewed_at ? formatDateTime(row.reviewed_at) : "\u2014",
        },
        ...(canManage
          ? [
              {
                key: "actions" as keyof WithdrawalDatatableRow,
                label: t("Actions"),
                sortable: false,
                render: (row: WithdrawalDatatableRow, ctx: DataTableCellContext<WithdrawalDatatableRow>) => {
                  // Only show actions for Pending or Processing
                  if (row.status !== "1" && row.status !== "2") return null;
                  return (
                    <div className="flex gap-1">
                      <Button size="xs" variant="primary" onClick={() => openReviewModal(row, ctx.refresh)}>
                        {t("Review")}
                      </Button>
                      <Button size="xs" variant="secondary" onClick={() => openUploadReceiptModal(row, ctx.refresh)}>
                        {t("Receipt")}
                      </Button>
                    </div>
                  );
                },
              },
            ]
          : []),
      ]}
    />
  );
}
