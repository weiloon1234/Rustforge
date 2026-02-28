import { Menu, LogOut } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@admin/stores/auth";

export default function Header({
  collapsed,
  onToggle,
}: {
  collapsed: boolean;
  onToggle: () => void;
}) {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);
  const logout = useAuthStore((s) => s.logout);

  return (
    <header className="rf-header">
      <button
        onClick={onToggle}
        className="rounded-lg p-2 text-muted transition-colors hover:bg-surface-hover hover:text-foreground"
        aria-label={collapsed ? t("Expand sidebar") : t("Collapse sidebar")}
      >
        <Menu size={20} />
      </button>

      <div className="flex-1" />

      <div className="flex items-center gap-3">
        <span className="text-sm text-muted">{account?.name ?? t("Admin")}</span>
        <button
          onClick={() => logout()}
          className="rounded-lg p-2 text-muted transition-colors hover:bg-surface-hover hover:text-foreground"
          aria-label={t("Logout")}
        >
          <LogOut size={18} />
        </button>
      </div>
    </header>
  );
}
