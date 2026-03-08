import { useTranslation } from "react-i18next";
import { useAuthStore } from "@user/stores/auth";

export default function DashboardPage() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold">
          {t("Welcome back, :name", { name: account?.name ?? account?.username ?? t("User") })}
        </h1>
        <p className="mt-1 text-sm text-muted">
          {t("Here's an overview of your account.")}
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <div className="rf-stat-card">
          <p className="text-xs font-semibold uppercase tracking-wide text-muted">
            {t("Member ID")}
          </p>
          <p className="mt-2 font-mono text-lg text-primary">
            {account?.uuid ?? "—"}
          </p>
        </div>

        <div className="rf-stat-card">
          <p className="text-xs font-semibold uppercase tracking-wide text-muted">
            {t("Username")}
          </p>
          <p className="mt-2 text-lg text-foreground">
            {account?.username ?? "—"}
          </p>
        </div>

        <div className="rf-stat-card">
          <p className="text-xs font-semibold uppercase tracking-wide text-muted">
            {t("Your referral link")}
          </p>
          <p className="mt-2 break-all font-mono text-sm text-primary">
            {account?.uuid ? `${window.location.origin}/register?ref=${account.uuid}` : "—"}
          </p>
        </div>
      </div>
    </div>
  );
}
