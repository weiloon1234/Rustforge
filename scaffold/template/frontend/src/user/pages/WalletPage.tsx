import { useTranslation } from "react-i18next";
import { Wallet } from "lucide-react";

export default function WalletPage() {
  const { t } = useTranslation();

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold">{t("Wallet")}</h1>
        <p className="mt-1 text-sm text-muted">
          {t("Manage your wallet and balances.")}
        </p>
      </div>

      <div className="flex flex-col items-center justify-center rounded-xl border border-border bg-surface py-16">
        <Wallet size={40} className="text-muted" />
        <p className="mt-4 text-sm text-muted">{t("Wallet coming soon.")}</p>
      </div>
    </div>
  );
}
