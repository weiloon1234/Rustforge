import { LayoutDashboard, Users, type LucideIcon } from "lucide-react";

export interface NavChild {
  label: string;
  path: string;
  permissions?: string[];
  notificationKey?: string;
}

export interface NavItem {
  label: string;
  icon: LucideIcon;
  path?: string;
  permissions?: string[];
  notificationKey?: string;
  children?: NavChild[];
}

/**
 * Centralized navigation config for the admin sidebar.
 *
 * To add a new page:
 *   1. Import the Lucide icon: `import { Settings } from "lucide-react";`
 *   2. Add an entry to this array.
 *   3. Create the page component in `pages/`.
 *   4. Add a `<Route>` in `App.tsx`.
 *
 * Permission strings match `app/permissions.toml` keys (e.g. "admin.read").
 * If `permissions` is omitted the item is visible to all authenticated admins.
 *
 * `notificationKey` connects to `useNotificationStore.counts` for badge display.
 * Parent items with children auto-sum their visible children's counts.
 */
export const navigation: NavItem[] = [
  {
    label: "Dashboard",
    icon: LayoutDashboard,
    path: "/",
  },
  {
    label: "Admins",
    icon: Users,
    path: "/admins",
    permissions: ["admin.read", "admin.manage"],
  },
];
