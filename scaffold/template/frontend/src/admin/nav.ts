import {
  LayoutDashboard,
  List,
  Settings,
  type LucideIcon,
} from "lucide-react";
import {
  ADMIN_TYPE,
  PERMISSION,
  type AdminType,
  type Permission,
} from "@admin/types";

export interface NavChild {
  label: string;
  path: string;
  permissions?: Permission[];
  admin_types?: AdminType[];
  notificationKey?: string;
}

export interface NavItem {
  label: string;
  icon: LucideIcon;
  path?: string;
  permissions?: Permission[];
  admin_types?: AdminType[];
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
    label: "Other",
    icon: Settings,
    children: [
      {
        label: "Admins",
        path: "/other/admins",
        permissions: [PERMISSION.ADMIN_READ, PERMISSION.ADMIN_MANAGE],
      },
      {
        label: "Pages",
        path: "/other/content-pages",
        permissions: [PERMISSION.CONTENT_PAGE_READ, PERMISSION.CONTENT_PAGE_MANAGE],
      },
      {
        label: "Countries",
        path: "/other/countries",
        permissions: [PERMISSION.COUNTRY_READ, PERMISSION.COUNTRY_MANAGE],
      },
    ],
  },
  {
    label: "Developer",
    icon: List,
    admin_types: [ADMIN_TYPE.DEVELOPER],
    children: [
      {
        label: "HTTP Client Logs",
        path: "/developer/http-client-logs",
        permissions: [PERMISSION.ADMIN_READ, PERMISSION.ADMIN_MANAGE],
        admin_types: [ADMIN_TYPE.DEVELOPER],
      },
      {
        label: "Webhook Logs",
        path: "/developer/webhook-logs",
        permissions: [PERMISSION.ADMIN_READ, PERMISSION.ADMIN_MANAGE],
        admin_types: [ADMIN_TYPE.DEVELOPER],
      },
    ],
  },
];
