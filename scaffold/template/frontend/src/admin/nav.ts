import {
  Banknote,
  LayoutDashboard,
  List,
  Settings,
  Users,
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
    label: "User",
    icon: Users,
    children: [
      {
        label: "Manage Users",
        path: "/user/manage",
        permissions: [PERMISSION.USER_READ, PERMISSION.USER_MANAGE],
      },
      {
        label: "User Hierarchy",
        path: "/user/hierarchy",
        permissions: [PERMISSION.USER_HIERARCHY],
      },
      {
        label: "Adjust Credits",
        path: "/user/adjust-credits",
        permissions: [PERMISSION.USER_CREDIT_MANAGE],
      },
      {
        label: "Introducer Changes",
        path: "/user/introducer-changes",
        permissions: [PERMISSION.USER_CHANGE_INTRODUCER],
      },
    ],
  },
  {
    label: "Finance",
    icon: Banknote,
    children: [
      {
        label: "Deposits",
        path: "/finance/deposits",
        permissions: [PERMISSION.DEPOSIT_READ, PERMISSION.DEPOSIT_MANAGE],
        notificationKey: "deposit",
      },
      {
        label: "Withdrawals",
        path: "/finance/withdrawals",
        permissions: [PERMISSION.WITHDRAWAL_READ, PERMISSION.WITHDRAWAL_MANAGE],
        notificationKey: "withdrawal",
      },
      {
        label: "Banks",
        path: "/finance/banks",
        permissions: [PERMISSION.BANK_READ, PERMISSION.BANK_MANAGE],
      },
      {
        label: "Crypto Networks",
        path: "/finance/crypto-networks",
        permissions: [PERMISSION.CRYPTO_NETWORK_READ, PERMISSION.CRYPTO_NETWORK_MANAGE],
      },
      {
        label: "Company Bank Accounts",
        path: "/finance/company-bank-accounts",
        permissions: [PERMISSION.COMPANY_BANK_ACCOUNT_READ, PERMISSION.COMPANY_BANK_ACCOUNT_MANAGE],
      },
      {
        label: "Company Crypto Accounts",
        path: "/finance/company-crypto-accounts",
        permissions: [PERMISSION.COMPANY_CRYPTO_ACCOUNT_READ, PERMISSION.COMPANY_CRYPTO_ACCOUNT_MANAGE],
      },
    ],
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
      {
        label: "Audit Logs",
        path: "/other/audit-logs",
        permissions: [PERMISSION.AUDIT_LOG_READ],
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
        admin_types: [ADMIN_TYPE.DEVELOPER],
      },
      {
        label: "Webhook Logs",
        path: "/developer/webhook-logs",
        admin_types: [ADMIN_TYPE.DEVELOPER],
      },
      {
        label: "SQL Profiler Requests",
        path: "/developer/sql-profiler-requests",
        admin_types: [ADMIN_TYPE.DEVELOPER],
      },
      {
        label: "SQL Profiler Queries",
        path: "/developer/sql-profiler-queries",
        admin_types: [ADMIN_TYPE.DEVELOPER],
      },
      {
        label: "Log Viewer",
        path: "/developer/log-viewer",
        admin_types: [ADMIN_TYPE.DEVELOPER],
      },
    ],
  },
];
