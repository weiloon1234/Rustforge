import { useLocation, Link } from "react-router-dom";
import { Home, History, Wallet, UserCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { LucideIcon } from "lucide-react";

export interface UserNavItem {
  label: string;
  path: string;
  icon: LucideIcon;
}

export const userNavigation: UserNavItem[] = [
  { label: "Home", path: "/", icon: Home },
  { label: "History", path: "/history", icon: History },
  { label: "Wallet", path: "/wallet", icon: Wallet },
  { label: "Me", path: "/me", icon: UserCircle },
];

function normalizePath(path: string): string {
  if (path === "/") return "/";
  return path.replace(/\/+$/, "") || "/";
}

function isPathActive(basePath: string, pathname: string): boolean {
  const base = normalizePath(basePath);
  const current = normalizePath(pathname);
  if (base === "/") return current === "/";
  return current === base || current.startsWith(`${base}/`);
}

export default function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();

  return (
    <aside className="rf-user-sidebar">
      <nav className="flex flex-col gap-1 p-3">
        {userNavigation.map((item) => {
          const active = isPathActive(item.path, location.pathname);
          const Icon = item.icon;
          return (
            <Link
              key={item.path}
              to={item.path}
              className={`rf-user-sidebar-link ${active ? "rf-user-sidebar-link-active" : ""}`}
            >
              <Icon size={20} className="shrink-0" />
              <span className="flex-1 truncate">{t(item.label)}</span>
            </Link>
          );
        })}
      </nav>
    </aside>
  );
}
