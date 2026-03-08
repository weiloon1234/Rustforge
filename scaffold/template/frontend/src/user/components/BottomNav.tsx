import { useLocation, Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { userNavigation } from "@user/components/Sidebar";

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

export default function BottomNav() {
  const { t } = useTranslation();
  const location = useLocation();

  return (
    <nav className="rf-bottom-nav">
      {userNavigation.map((item) => {
        const active = isPathActive(item.path, location.pathname);
        const Icon = item.icon;
        return (
          <Link
            key={item.path}
            to={item.path}
            className={`rf-bottom-nav-item ${active ? "rf-bottom-nav-item-active" : ""}`}
          >
            <Icon size={22} />
            <span>{t(item.label)}</span>
          </Link>
        );
      })}
    </nav>
  );
}
