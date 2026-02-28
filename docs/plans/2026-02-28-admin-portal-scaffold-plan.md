# Admin Portal Scaffold Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the scaffold to generate a complete, ready-to-use admin portal with login screen, collapsible sidebar layout, dashboard, and admins page — all wired and working out of the box.

**Architecture:** All changes are in `scaffold/src/templates.rs` (new/modified template constants) and `scaffold/src/main.rs` (register new files). The scaffold generates static files — no runtime logic changes. Frontend uses React + react-router-dom + Zustand + Tailwind CSS 4 + Lucide React icons.

**Tech Stack:** Rust (scaffold CLI), React 19, TypeScript, Tailwind CSS 4, Zustand 5, Lucide React, react-router-dom 7

**Design doc:** `docs/plans/2026-02-28-admin-portal-scaffold-design.md`

---

### Task 1: Add `lucide-react` to package.json template

**Files:**
- Modify: `scaffold/src/templates.rs` — `FRONTEND_PACKAGE_JSON` constant (line ~4458)

**Step 1: Add lucide-react dependency**

In the `FRONTEND_PACKAGE_JSON` constant, add `"lucide-react": "^0.468.0"` to the `"dependencies"` block (after the `"i18next"` line, alphabetical order).

**Step 2: Verify scaffold compiles**

Run: `cd /Users/weiloonso/Projects/personal/Rust/Rustforge && cargo check -p scaffold`
Expected: compiles successfully

**Step 3: Commit**

```bash
git add scaffold/src/templates.rs
git commit -m "feat(scaffold): add lucide-react dependency to package.json template"
```

---

### Task 2: Extend admin `app.css` with layout component classes

**Files:**
- Modify: `scaffold/src/templates.rs` — `FRONTEND_SRC_ADMIN_APP_CSS` constant (line ~4841)

**Step 1: Add component classes**

Append these classes inside the existing `@layer components { ... }` block, after the existing `.rf-form-grid` rule:

```css
  /* ── Layout ─────────────────────────────────────── */
  .rf-sidebar {
    @apply fixed left-0 top-14 bottom-0 bg-surface border-r border-border
      transition-all duration-200 overflow-y-auto overflow-x-hidden z-20;
  }
  .rf-sidebar-expanded { @apply w-64; }
  .rf-sidebar-collapsed { @apply w-16; }
  .rf-sidebar-link {
    @apply flex items-center gap-3 px-4 py-2.5 text-sm text-muted rounded-lg
      transition-colors duration-150 hover:bg-surface-hover hover:text-foreground whitespace-nowrap;
  }
  .rf-sidebar-link-active {
    @apply bg-primary/10 text-primary hover:bg-primary/15 hover:text-primary;
  }
  .rf-header {
    @apply fixed top-0 left-0 right-0 h-14 bg-surface border-b border-border
      flex items-center px-4 z-30;
  }
  .rf-stat-card {
    @apply rounded-xl bg-surface border border-border p-5;
  }
  .rf-badge {
    @apply inline-flex items-center justify-center min-w-5 h-5 px-1.5 text-xs
      font-semibold rounded-full bg-primary text-primary-foreground;
  }
```

**Step 2: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 3: Commit**

```bash
git add scaffold/src/templates.rs
git commit -m "feat(scaffold): add sidebar/header/stat/badge CSS component classes to admin theme"
```

---

### Task 3: Create notification store stub template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_STORES_NOTIFICATIONS_TS`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

Add this constant in `templates.rs` near the existing `FRONTEND_SRC_ADMIN_STORES_AUTH_TS` constant (line ~5788):

```rust
pub const FRONTEND_SRC_ADMIN_STORES_NOTIFICATIONS_TS: &str = r#"import { create } from "zustand";

interface NotificationState {
  /** Map of notification keys to their pending counts. */
  counts: Record<string, number>;
  /** Get the count for a given key (returns 0 if not set). */
  getCount: (key: string) => number;
  /** Set count for a key. Call this from your polling/websocket handler. */
  setCount: (key: string, count: number) => void;
  /** Batch-set multiple counts at once. */
  setCounts: (counts: Record<string, number>) => void;
}

export const useNotificationStore = create<NotificationState>()((set, get) => ({
  counts: {},
  getCount: (key) => get().counts[key] ?? 0,
  setCount: (key, count) =>
    set((state) => ({ counts: { ...state.counts, [key]: count } })),
  setCounts: (counts) =>
    set((state) => ({ counts: { ...state.counts, ...counts } })),
}));
"#;
```

**Step 2: Register in main.rs**

Add a `FileTemplate` entry after the existing admin `stores/auth.ts` registration (line ~662):

```rust
        FileTemplate {
            path: "frontend/src/admin/stores/notifications.ts",
            content: templates::FRONTEND_SRC_ADMIN_STORES_NOTIFICATIONS_TS,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add notification store stub for sidebar badge counts"
```

---

### Task 4: Create nav config template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_NAV_TS`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

```rust
pub const FRONTEND_SRC_ADMIN_NAV_TS: &str = r#"import { LayoutDashboard, Users, type LucideIcon } from "lucide-react";

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
"#;
```

**Step 2: Register in main.rs**

Add a `FileTemplate` entry after the admin `App.tsx` registration (line ~535):

```rust
        FileTemplate {
            path: "frontend/src/admin/nav.ts",
            content: templates::FRONTEND_SRC_ADMIN_NAV_TS,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add centralized nav config with permission and notification support"
```

---

### Task 5: Create Sidebar component template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_COMPONENTS_SIDEBAR_TSX`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

```rust
pub const FRONTEND_SRC_ADMIN_COMPONENTS_SIDEBAR_TSX: &str = r#"import { useLocation, Link } from "react-router-dom";
import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { navigation, type NavItem, type NavChild } from "../nav";
import { useAuthStore } from "../stores/auth";
import { useNotificationStore } from "../stores/notifications";

function hasAccess(scopes: string[], required?: string[]): boolean {
  if (!required || required.length === 0) return true;
  return required.some((p) => scopes.includes(p));
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
  const count = useNotificationStore((s) => s.getCount(item.notificationKey ?? ""));
  const Icon = item.icon;

  return (
    <Link
      to={item.path}
      className={`rf-sidebar-link ${active ? "rf-sidebar-link-active" : ""}`}
      title={collapsed ? item.label : undefined}
    >
      {Icon && <Icon size={20} className="shrink-0" />}
      {!collapsed && (
        <>
          <span className="flex-1 truncate">{item.label}</span>
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
}: {
  item: NavItem;
  collapsed: boolean;
  scopes: string[];
}) {
  const location = useLocation();
  const [open, setOpen] = useState(false);
  const getCount = useNotificationStore((s) => s.getCount);

  const visibleChildren = (item.children ?? []).filter((c) =>
    hasAccess(scopes, c.permissions),
  );

  const totalCount = visibleChildren.reduce(
    (sum, c) => sum + getCount(c.notificationKey ?? ""),
    0,
  );

  const isChildActive = visibleChildren.some(
    (c) => location.pathname === c.path,
  );

  const Icon = item.icon;

  if (collapsed) {
    return (
      <div className="relative" title={item.label}>
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
        <span className="flex-1 truncate text-left">{item.label}</span>
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

  const visibleItems = navigation.filter((item) => {
    if (!hasAccess(scopes, item.permissions)) return false;
    if (item.children) {
      return item.children.some((c) => hasAccess(scopes, c.permissions));
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
"#;
```

**Step 2: Register in main.rs**

```rust
        FileTemplate {
            path: "frontend/src/admin/components/Sidebar.tsx",
            content: templates::FRONTEND_SRC_ADMIN_COMPONENTS_SIDEBAR_TSX,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add Sidebar component with permission filtering and notification badges"
```

---

### Task 6: Create Header component template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_COMPONENTS_HEADER_TSX`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

```rust
pub const FRONTEND_SRC_ADMIN_COMPONENTS_HEADER_TSX: &str = r#"import { Menu, LogOut } from "lucide-react";
import { useAuthStore } from "../stores/auth";

export default function Header({
  collapsed,
  onToggle,
}: {
  collapsed: boolean;
  onToggle: () => void;
}) {
  const account = useAuthStore((s) => s.account);
  const logout = useAuthStore((s) => s.logout);

  return (
    <header className="rf-header">
      <button
        onClick={onToggle}
        className="rounded-lg p-2 text-muted transition-colors hover:bg-surface-hover hover:text-foreground"
        aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
      >
        <Menu size={20} />
      </button>

      <div className="flex-1" />

      <div className="flex items-center gap-3">
        <span className="text-sm text-muted">{account?.name ?? "Admin"}</span>
        <button
          onClick={() => logout()}
          className="rounded-lg p-2 text-muted transition-colors hover:bg-surface-hover hover:text-foreground"
          aria-label="Logout"
        >
          <LogOut size={18} />
        </button>
      </div>
    </header>
  );
}
"#;
```

**Step 2: Register in main.rs**

```rust
        FileTemplate {
            path: "frontend/src/admin/components/Header.tsx",
            content: templates::FRONTEND_SRC_ADMIN_COMPONENTS_HEADER_TSX,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add Header component with sidebar toggle and logout"
```

---

### Task 7: Create AdminLayout template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_LAYOUTS_ADMIN_LAYOUT_TSX`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

```rust
pub const FRONTEND_SRC_ADMIN_LAYOUTS_ADMIN_LAYOUT_TSX: &str = r#"import { useState, useEffect } from "react";
import { Outlet } from "react-router-dom";
import Sidebar from "../components/Sidebar";
import Header from "../components/Header";

const STORAGE_KEY = "admin-sidebar-collapsed";

export default function AdminLayout() {
  const [collapsed, setCollapsed] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, String(collapsed));
  }, [collapsed]);

  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header collapsed={collapsed} onToggle={() => setCollapsed((c) => !c)} />
      <Sidebar collapsed={collapsed} />
      <main
        className="pt-14 transition-all duration-200"
        style={{ marginLeft: collapsed ? "4rem" : "16rem" }}
      >
        <div className="p-6">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
"#;
```

**Step 2: Register in main.rs**

```rust
        FileTemplate {
            path: "frontend/src/admin/layouts/AdminLayout.tsx",
            content: templates::FRONTEND_SRC_ADMIN_LAYOUTS_ADMIN_LAYOUT_TSX,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add AdminLayout with collapsible sidebar and persistent state"
```

---

### Task 8: Create LoginPage template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_PAGES_LOGIN_PAGE_TSX`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

```rust
pub const FRONTEND_SRC_ADMIN_PAGES_LOGIN_PAGE_TSX: &str = r#"import { useNavigate } from "react-router-dom";
import { useAutoForm } from "../../shared/useAutoForm";
import { useAuthStore } from "../stores/auth";
import { api } from "../api";

export default function LoginPage() {
  const navigate = useNavigate();
  const setToken = useAuthStore((s) => s.setToken);
  const fetchAccount = useAuthStore((s) => s.fetchAccount);

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "/api/v1/admin/auth/login",
    method: "post",
    fields: [
      {
        name: "username",
        type: "text",
        label: "Username",
        placeholder: "Enter your username",
        required: true,
        span: 2,
      },
      {
        name: "password",
        type: "password",
        label: "Password",
        placeholder: "Enter your password",
        required: true,
        span: 2,
      },
    ],
    onSuccess: async (data: unknown) => {
      const result = data as { access_token: string };
      setToken(result.access_token);
      await fetchAccount();
      navigate("/");
    },
  });

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4">
      <div className="w-full max-w-sm">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Admin Portal
          </h1>
          <p className="mt-1 text-sm text-muted">
            Sign in to your account
          </p>
        </div>

        <div className="rounded-xl border border-border bg-surface p-6">
          {errors.general && (
            <div className="mb-4 rounded-lg bg-error-muted px-3 py-2 text-sm text-error">
              {errors.general}
            </div>
          )}

          {form}

          <button
            onClick={submit}
            disabled={busy}
            className="mt-2 w-full rounded-lg bg-primary px-4 py-2.5 text-sm font-medium
              text-primary-foreground transition-colors hover:bg-primary-hover
              disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {busy ? "Signing in..." : "Sign in"}
          </button>
        </div>
      </div>
    </div>
  );
}
"#;
```

**Step 2: Register in main.rs**

```rust
        FileTemplate {
            path: "frontend/src/admin/pages/LoginPage.tsx",
            content: templates::FRONTEND_SRC_ADMIN_PAGES_LOGIN_PAGE_TSX,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add LoginPage using useAutoForm with admin auth"
```

---

### Task 9: Create DashboardPage template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_PAGES_DASHBOARD_PAGE_TSX`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

```rust
pub const FRONTEND_SRC_ADMIN_PAGES_DASHBOARD_PAGE_TSX: &str = r#"import { Users, Activity, UserPlus, ShieldCheck } from "lucide-react";
import { useAuthStore } from "../stores/auth";

const stats = [
  { label: "Total Admins", value: "—", icon: Users },
  { label: "Active Today", value: "—", icon: Activity },
  { label: "New This Week", value: "—", icon: UserPlus },
  { label: "System Health", value: "OK", icon: ShieldCheck },
];

export default function DashboardPage() {
  const account = useAuthStore((s) => s.account);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-foreground">
          Welcome back, {account?.name ?? "Admin"}
        </h1>
        <p className="mt-1 text-sm text-muted">
          Here's an overview of your system.
        </p>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <div key={stat.label} className="rf-stat-card">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-primary/10 p-2">
                  <Icon size={20} className="text-primary" />
                </div>
                <div>
                  <p className="text-xs text-muted">{stat.label}</p>
                  <p className="text-lg font-semibold text-foreground">
                    {stat.value}
                  </p>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
"#;
```

**Step 2: Register in main.rs**

```rust
        FileTemplate {
            path: "frontend/src/admin/pages/DashboardPage.tsx",
            content: templates::FRONTEND_SRC_ADMIN_PAGES_DASHBOARD_PAGE_TSX,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add DashboardPage with welcome banner and stat cards"
```

---

### Task 10: Create AdminsPage template

**Files:**
- Modify: `scaffold/src/templates.rs` — add new constant `FRONTEND_SRC_ADMIN_PAGES_ADMINS_PAGE_TSX`
- Modify: `scaffold/src/main.rs` — register the new file

**Step 1: Add template constant**

```rust
pub const FRONTEND_SRC_ADMIN_PAGES_ADMINS_PAGE_TSX: &str = r#"import { Users } from "lucide-react";

export default function AdminsPage() {
  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-foreground">Admins</h1>
        <p className="mt-1 text-sm text-muted">
          Manage administrator accounts
        </p>
      </div>

      <div className="rounded-xl border border-border bg-surface p-8 text-center">
        <div className="mx-auto mb-3 w-fit rounded-lg bg-primary/10 p-3">
          <Users size={24} className="text-primary" />
        </div>
        <p className="text-sm text-muted">
          Connect to the admin datatable API to display data here.
        </p>
        <p className="mt-1 text-xs text-muted-foreground">
          See <code className="rounded bg-surface-hover px-1.5 py-0.5 text-foreground">types/datatable-admin.ts</code> for the query contract.
        </p>
      </div>
    </div>
  );
}
"#;
```

**Step 2: Register in main.rs**

```rust
        FileTemplate {
            path: "frontend/src/admin/pages/AdminsPage.tsx",
            content: templates::FRONTEND_SRC_ADMIN_PAGES_ADMINS_PAGE_TSX,
            executable: false,
        },
```

**Step 3: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 4: Commit**

```bash
git add scaffold/src/templates.rs scaffold/src/main.rs
git commit -m "feat(scaffold): add AdminsPage placeholder with datatable integration guide"
```

---

### Task 11: Rewrite admin `App.tsx` template

**Files:**
- Modify: `scaffold/src/templates.rs` — replace existing `FRONTEND_SRC_ADMIN_APP_TSX` constant (line ~4803)

**Step 1: Replace the constant**

Replace the entire `FRONTEND_SRC_ADMIN_APP_TSX` constant with:

```rust
pub const FRONTEND_SRC_ADMIN_APP_TSX: &str = r#"import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "../shared/ProtectedRoute";
import { useAuthStore } from "./stores/auth";
import AdminLayout from "./layouts/AdminLayout";
import LoginPage from "./pages/LoginPage";
import DashboardPage from "./pages/DashboardPage";
import AdminsPage from "./pages/AdminsPage";

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
        <Route element={<AdminLayout />}>
          <Route index element={<DashboardPage />} />
          <Route path="/admins" element={<AdminsPage />} />
        </Route>
      </Route>
    </Routes>
  );
}
"#;
```

**Step 2: Verify scaffold compiles**

Run: `cargo check -p scaffold`

**Step 3: Commit**

```bash
git add scaffold/src/templates.rs
git commit -m "feat(scaffold): rewrite admin App.tsx with layout routes and page imports"
```

---

### Task 12: Final verification

**Step 1: Full build check**

Run: `cargo check -p scaffold`

**Step 2: Dry-run scaffold to temp directory**

Run:
```bash
cd /Users/weiloonso/Projects/personal/Rust/Rustforge
cargo run -p scaffold -- --output /tmp/rustforge-test --force
```

**Step 3: Verify generated files exist**

Check that all new files were created:
```bash
ls -la /tmp/rustforge-test/frontend/src/admin/nav.ts
ls -la /tmp/rustforge-test/frontend/src/admin/layouts/AdminLayout.tsx
ls -la /tmp/rustforge-test/frontend/src/admin/components/Sidebar.tsx
ls -la /tmp/rustforge-test/frontend/src/admin/components/Header.tsx
ls -la /tmp/rustforge-test/frontend/src/admin/pages/LoginPage.tsx
ls -la /tmp/rustforge-test/frontend/src/admin/pages/DashboardPage.tsx
ls -la /tmp/rustforge-test/frontend/src/admin/pages/AdminsPage.tsx
ls -la /tmp/rustforge-test/frontend/src/admin/stores/notifications.ts
```

**Step 4: Verify package.json has lucide-react**

```bash
grep lucide /tmp/rustforge-test/frontend/package.json
```

Expected: `"lucide-react": "^0.468.0"`

**Step 5: Clean up**

```bash
rm -rf /tmp/rustforge-test
```

**Step 6: Commit (if any fixes were needed)**

```bash
git add scaffold/
git commit -m "fix(scaffold): address issues found in dry-run verification"
```
