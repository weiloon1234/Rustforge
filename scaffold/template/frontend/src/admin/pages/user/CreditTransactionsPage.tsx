import { useTranslation } from "react-i18next";
import type { UserCreditTransactionDatatableRow } from "@admin/types";
import { CREDIT_TYPE_I18N } from "@admin/constants/enums";
import {
  DataTable,
  formatDateTime,
} from "@shared/components";

export default function CreditTransactionsPage() {
  const { t } = useTranslation();

  return (
    <DataTable<UserCreditTransactionDatatableRow>
      url="datatable/user_credit_transaction/query"
      title={t("Credit Transactions")}
      subtitle={t("View credit transaction records")}
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
