import { useLocation, Link } from "react-router-dom";
import { ChevronDown } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { navigation, type NavItem, type NavChild } from "@admin/nav";
import type { AdminType, Permission } from "@admin/types";
import { useAuthStore } from "@admin/stores/auth";
import { useNotificationStore } from "@admin/stores/notifications";

function matchPattern(pattern: string, value: string): boolean {
  if (!pattern.endsWith(".*")) return false;
  const prefix = pattern.slice(0, -2);
  if (!prefix) return false;
  return value === prefix || value.startsWith(prefix + ".");
}

function manageImpliesRead(granted: string, required: string): boolean {
  const gi = granted.lastIndexOf(".");
  const ri = required.lastIndexOf(".");
  if (gi === -1 || ri === -1) return false;
  return (
    granted.slice(0, gi) === required.slice(0, ri) &&
    granted.slice(gi + 1) === "manage" &&
    required.slice(ri + 1) === "read"
  );
}

function permissionMatches(granted: string, required: string): boolean {
  const g = granted.trim();
  const r = required.trim();
  if (!g || !r) return false;
  if (g === "*" || r === "*" || g === r) return true;
  if (manageImpliesRead(g, r)) return true;
  return matchPattern(g, r) || matchPattern(r, g);
}

function hasAccess(scopes: readonly string[], required?: readonly Permission[]): boolean {
  if (!required || required.length === 0) return true;
  return required.some((r) => scopes.some((g) => permissionMatches(g, r)));
}

function hasAdminTypeAccess(
  adminType: AdminType | null,
  required?: readonly AdminType[],
): boolean {
  if (!required || required.length === 0) return true;
  if (!adminType) return false;
  return required.includes(adminType);
}

function Badge({ count }: { count: number }) {
  if (count <= 0) return null;
  return <span className="rf-badge">{count > 99 ? "99+" : count}</span>;
}

function NavLink({
  item,
  active,
  collapsed,
}: {
  item: { label: string; path: string; icon?: NavItem["icon"]; notificationKey?: string };
  active: boolean;
  collapsed: boolean;
}) {
  const { t } = useTranslation();
  const count = useNotificationStore((s) => s.getCount(item.notificationKey ?? ""));
  const Icon = item.icon;

  return (
    <Link
      to={item.path}
      className={`rf-sidebar-link ${active ? "rf-sidebar-link-active" : ""}`}
      title={collapsed ? t(item.label) : undefined}
    >
      {Icon && <Icon size={20} className="shrink-0" />}
      {!collapsed && (
        <>
          <span className="flex-1 truncate">{t(item.label)}</span>
          <Badge count={count} />
        </>
      )}
      {collapsed && count > 0 && (
        <span className="absolute right-1 top-1 h-2 w-2 rounded-full bg-primary" />
      )}
    </Link>
  );
}

function ParentNav({
  item,
  collapsed,
  scopes,
  adminType,
}: {
  item: NavItem;
  collapsed: boolean;
  scopes: string[];
  adminType: AdminType | null;
}) {
  const { t } = useTranslation();
  const location = useLocation();
  const [open, setOpen] = useState(false);
  const getCount = useNotificationStore((s) => s.getCount);

  const visibleChildren = (item.children ?? []).filter((c) =>
    hasAccess(scopes, c.permissions) &&
    hasAdminTypeAccess(adminType, c.admin_types),
  );

  const totalCount = visibleChildren.reduce(
    (sum, c) => sum + getCount(c.notificationKey ?? ""),
    0,
  );

  const isChildActive = visibleChildren.some(
    (c) => location.pathname === c.path,
  );

  useEffect(() => {
    if (isChildActive) {
      setOpen(true);
    }
  }, [isChildActive]);

  const Icon = item.icon;

  if (collapsed) {
    return (
      <div className="relative" title={t(item.label)}>
        <button
          className={`rf-sidebar-link w-full ${isChildActive ? "rf-sidebar-link-active" : ""}`}
          onClick={() => setOpen(!open)}
        >
          <Icon size={20} className="shrink-0" />
          {totalCount > 0 && (
            <span className="absolute right-1 top-1 h-2 w-2 rounded-full bg-primary" />
          )}
        </button>
      </div>
    );
  }

  return (
    <div>
      <button
        className={`rf-sidebar-link w-full ${isChildActive ? "rf-sidebar-link-active" : ""}`}
        onClick={() => setOpen(!open)}
      >
        <Icon size={20} className="shrink-0" />
        <span className="flex-1 truncate text-left">{t(item.label)}</span>
        <Badge count={totalCount} />
        <ChevronDown
          size={16}
          className={`shrink-0 transition-transform duration-150 ${open ? "rotate-180" : ""}`}
        />
      </button>
      {open && (
        <div className="ml-7 mt-0.5 space-y-0.5">
          {visibleChildren.map((child) => (
            <NavLink
              key={child.path}
              item={child}
              active={location.pathname === child.path}
              collapsed={false}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default function Sidebar({ collapsed }: { collapsed: boolean }) {
  const location = useLocation();
  const scopes = useAuthStore((s) => s.account?.scopes ?? []);
  const adminType = useAuthStore((s) => s.account?.admin_type ?? null);

  const visibleItems = navigation.filter((item) => {
    if (!hasAccess(scopes, item.permissions)) return false;
    if (!hasAdminTypeAccess(adminType, item.admin_types)) return false;
    if (item.children) {
      return item.children.some(
        (c) =>
          hasAccess(scopes, c.permissions) &&
          hasAdminTypeAccess(adminType, c.admin_types),
      );
    }
    return true;
  });

  return (
    <aside className={`rf-sidebar ${collapsed ? "rf-sidebar-collapsed" : "rf-sidebar-expanded"}`}>
      <nav className="flex flex-col gap-1 p-3">
        {visibleItems.map((item) => {
          if (item.children) {
            return (
              <ParentNav
                key={item.label}
                item={item}
                collapsed={collapsed}
                scopes={scopes}
                adminType={adminType}
              />
            );
          }

          return (
            <NavLink
              key={item.path!}
              item={{ ...item, path: item.path!, icon: item.icon }}
              active={location.pathname === item.path}
              collapsed={collapsed}
            />
          );
        })}
      </nav>
    </aside>
  );
}
