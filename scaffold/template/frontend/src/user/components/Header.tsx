import { Gamepad2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@user/stores/auth";

export default function Header() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);

  return (
    <header className="rf-header">
      <div className="flex items-center gap-2 text-primary">
        <Gamepad2 size={20} />
        <span className="text-sm font-semibold tracking-wide text-foreground">
          {t("User Portal")}
        </span>
      </div>

      <div className="flex-1" />

      <span className="text-sm text-muted">
        {account?.name ?? account?.username ?? ""}
      </span>
    </header>
  );
}
