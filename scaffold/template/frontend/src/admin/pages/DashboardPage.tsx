import { useTranslation } from "react-i18next";
import { useAuthStore } from "@admin/stores/auth";

export default function DashboardPage() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-foreground">
          {t("Welcome back, :name", { name: account?.name ?? t("Admin") })}
        </h1>
        <p className="mt-1 text-sm text-muted">
          {t("Here's an overview of your system.")}
        </p>
      </div>
    </div>
  );
}
