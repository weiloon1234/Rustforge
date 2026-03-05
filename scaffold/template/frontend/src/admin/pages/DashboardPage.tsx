import { useTranslation } from "react-i18next";
import { useAuthStore } from "@admin/stores/auth";

export default function DashboardPage() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);
  const appName = import.meta.env.VITE_APP_NAME?.trim() || "starter";

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold">
          {t("Welcome back, :name", { name: account?.name ?? t("Admin") })}
        </h1>
        <p className="mt-1 text-sm text-muted">
          {t("Here's an overview of your system.")}
        </p>
        <p className="mt-2 text-sm text-muted">
          {t("Hello, Welcome to :app_name", { app_name: appName })}
        </p>
      </div>
    </div>
  );
}
