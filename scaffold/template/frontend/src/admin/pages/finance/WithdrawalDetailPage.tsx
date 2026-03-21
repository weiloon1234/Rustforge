import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { ArrowLeft, Loader2 } from "lucide-react";
import { api } from "@admin/api";
import { CREDIT_TYPE_I18N, WITHDRAWAL_METHOD_I18N, WITHDRAWAL_STATUS, WITHDRAWAL_STATUS_I18N } from "@admin/types";
import type { WithdrawalOutput, WithdrawalStatus } from "@admin/types";
import {
  Button,
  DataTable,
  moneyFormat,
  formatDateTime,
} from "@shared/components";
import type { ApiResponse } from "@shared/types";

type TabKey = "credit_transactions" | "deposits";

function statusColorFor(status: WithdrawalStatus): string {
  switch (status) {
    case WITHDRAWAL_STATUS.PENDING: return "bg-yellow-100 text-yellow-800";
    case WITHDRAWAL_STATUS.PROCESSING: return "bg-blue-100 text-blue-800";
    case WITHDRAWAL_STATUS.APPROVED: return "bg-green-100 text-green-800";
    case WITHDRAWAL_STATUS.REJECTED: return "bg-red-100 text-red-800";
  }
  return "bg-gray-100 text-gray-800";
}

export default function WithdrawalDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [withdrawal, setWithdrawal] = useState<WithdrawalOutput | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<TabKey>("credit_transactions");

  useEffect(() => {
    if (!id) return;
    api
      .get<ApiResponse<WithdrawalOutput>>(`/withdrawals/${id}`)
      .then((res) => setWithdrawal(res.data.data))
      .catch(() => setWithdrawal(null))
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-16">
        <Loader2 size={32} className="animate-spin text-muted" />
      </div>
    );
  }

  if (!withdrawal) {
    return (
      <div className="text-center py-16 text-muted">
        {t("Withdrawal not found")}
      </div>
    );
  }

  const creditTypeStorage = String(withdrawal.credit_type);
  const ownerIdStr = String(withdrawal.owner_id);

  const tabs: { key: TabKey; label: string }[] = [
    { key: "credit_transactions", label: t("Credit Transactions") },
    { key: "deposits", label: t("Deposits") },
  ];

  return (
    <div>
      {/* Header */}
      <div className="mb-6 flex items-center gap-3">
        <Button
          variant="secondary"
          size="sm"
          onClick={() => navigate("/finance/withdrawals")}
        >
          <ArrowLeft size={16} />
        </Button>
        <div>
          <h1 className="text-2xl font-bold">
            {t("Withdrawal")} #{id}
          </h1>
        </div>
      </div>

      {/* Withdrawal info card */}
      <div className="mb-6 rounded-xl border border-border bg-surface p-4">
        <div className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm md:grid-cols-4">
          <div>
            <span className="text-muted">{t("Amount")}</span>
            <p className="font-medium">{moneyFormat(parseFloat(withdrawal.amount))}</p>
          </div>
          <div>
            <span className="text-muted">{t("Fee")}</span>
            <p className="font-medium">{moneyFormat(parseFloat(withdrawal.fee))}</p>
          </div>
          <div>
            <span className="text-muted">{t("Net Amount")}</span>
            <p className="font-medium">{moneyFormat(parseFloat(withdrawal.net_amount))}</p>
          </div>
          <div>
            <span className="text-muted">{t("Credit Type")}</span>
            <p className="font-medium">
              {t(CREDIT_TYPE_I18N[withdrawal.credit_type] ?? String(withdrawal.credit_type))}
            </p>
          </div>
          <div>
            <span className="text-muted">{t("Status")}</span>
            <p>
              <span
                className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${statusColorFor(withdrawal.status)}`}
              >
                {t(WITHDRAWAL_STATUS_I18N[withdrawal.status] ?? "Unknown")}
              </span>
            </p>
          </div>
          <div>
            <span className="text-muted">{t("Method")}</span>
            <p className="font-medium">
              {t(WITHDRAWAL_METHOD_I18N[withdrawal.withdrawal_method] ?? String(withdrawal.withdrawal_method))}
            </p>
          </div>
          <div>
            <span className="text-muted">{t("Destination")}</span>
            <p className="font-medium truncate">
              {withdrawal.crypto_wallet_address ?? withdrawal.bank_account_number ?? "\u2014"}
            </p>
          </div>
          <div>
            <span className="text-muted">{t("Created At")}</span>
            <p className="font-medium">{formatDateTime(withdrawal.created_at)}</p>
          </div>
        </div>
      </div>

      {/* Tab bar */}
      <div className="mb-4 flex gap-2">
        {tabs.map((tab) => (
          <Button
            key={tab.key}
            variant={activeTab === tab.key ? "primary" : "secondary"}
            size="sm"
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </Button>
        ))}
      </div>

      {/* Tab content — datatables filtered by credit_type + user_id */}
      {activeTab === "credit_transactions" && (
        <DataTable
          key={`ct-${id}`}
          url="datatable/user_credit_transaction/query"
          initialFilters={{
            "f-credit_type": creditTypeStorage,
            "f-user_id": ownerIdStr,
          }}
          showRefresh
          enableAutoRefresh={false}
          columns={[
            { key: "id", label: t("ID"), cellClassName: "tabular-nums text-muted" },
            { key: "user_username", label: t("User") },
            { key: "credit_type", label: t("Credit Type"), render: (row: Record<string, unknown>) => t(CREDIT_TYPE_I18N[String(row.credit_type) as keyof typeof CREDIT_TYPE_I18N] ?? String(row.credit_type)) },
            { key: "amount", label: t("Amount"), cellClassName: "tabular-nums", render: (row: Record<string, unknown>) => moneyFormat(parseFloat(String(row.amount))) },
            { key: "transaction_type_explained", label: t("Transaction Type") },
            { key: "related_key", label: t("Related Key"), cellClassName: "text-muted" },
            { key: "created_at", label: t("Created At"), cellClassName: "tabular-nums text-muted", render: (row: Record<string, unknown>) => formatDateTime(String(row.created_at)) },
          ]}
        />
      )}
      {activeTab === "deposits" && (
        <DataTable
          key={`dep-${id}`}
          url="datatable/deposit/query"
          initialFilters={{
            "f-credit_type": creditTypeStorage,
            "f-user_id": ownerIdStr,
          }}
          showRefresh
          enableAutoRefresh={false}
          columns={[
            { key: "id", label: t("ID"), cellClassName: "tabular-nums text-muted" },
            { key: "owner_id", label: t("Owner"), render: (row: Record<string, unknown>) => String(row.owner_name ?? row.owner_id) },
            { key: "credit_type", label: t("Credit Type"), render: (row: Record<string, unknown>) => t(CREDIT_TYPE_I18N[String(row.credit_type) as keyof typeof CREDIT_TYPE_I18N] ?? String(row.credit_type)) },
            { key: "amount", label: t("Amount"), cellClassName: "tabular-nums", render: (row: Record<string, unknown>) => moneyFormat(parseFloat(String(row.amount))) },
            { key: "fee", label: t("Fee"), cellClassName: "tabular-nums text-muted", render: (row: Record<string, unknown>) => moneyFormat(parseFloat(String(row.fee))) },
            { key: "net_amount", label: t("Net"), cellClassName: "tabular-nums", render: (row: Record<string, unknown>) => moneyFormat(parseFloat(String(row.net_amount))) },
            { key: "status_label", label: t("Status") },
            { key: "created_at", label: t("Created At"), cellClassName: "tabular-nums text-muted", render: (row: Record<string, unknown>) => formatDateTime(String(row.created_at)) },
          ]}
        />
      )}
    </div>
  );
}
