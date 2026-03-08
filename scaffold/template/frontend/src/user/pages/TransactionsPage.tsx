import { useTranslation } from "react-i18next";
import { History } from "lucide-react";

export default function TransactionsPage() {
  const { t } = useTranslation();

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold">{t("History")}</h1>
        <p className="mt-1 text-sm text-muted">
          {t("View your transaction history.")}
        </p>
      </div>

      <div className="flex flex-col items-center justify-center rounded-xl border border-border bg-surface py-16">
        <History size={40} className="text-muted" />
        <p className="mt-4 text-sm text-muted">{t("No transactions yet.")}</p>
      </div>
    </div>
  );
}
