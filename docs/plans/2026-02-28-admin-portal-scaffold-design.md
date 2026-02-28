# Admin Portal Scaffold Design

**Date:** 2026-02-28
**Status:** Approved

## Goal

Upgrade the scaffold to generate a complete, ready-to-use admin portal instead of placeholder stubs. After `cargo run -p scaffold`, the team gets a working login screen, authenticated layout with collapsible sidebar, dashboard, and admins page — all wired up.

## Decisions

- **Theme**: Keep existing dark slate + purple accent (already in `app.css`)
- **Sidebar**: Collapsible (w-64 expanded, w-16 collapsed), state in localStorage
- **Nav pages**: Dashboard (welcome + stat cards) + Admins (placeholder)
- **Icons**: Lucide React (tree-shakeable, modern)
- **Login**: Uses shared `useAutoForm` hook with existing auth store
- **Notifications**: Nav config supports `notificationKey` for future badge counts

## File Structure (Generated)

```
frontend/src/admin/
├── main.tsx                         (unchanged)
├── App.tsx                          (rewritten — routes + layout import)
├── app.css                          (extended — sidebar/header/stat component classes)
├── api.ts                           (unchanged)
├── nav.ts                           (NEW — centralized navigation config)
├── stores/
│   ├── auth.ts                      (unchanged)
│   └── notifications.ts            (NEW — stub notification count store)
├── types/...                        (unchanged)
├── layouts/
│   └── AdminLayout.tsx              (NEW — header + sidebar + content outlet)
├── components/
│   ├── Sidebar.tsx                  (NEW — collapsible sidebar with nav + badges)
│   └── Header.tsx                   (NEW — top bar with toggle + user menu)
└── pages/
    ├── LoginPage.tsx                (NEW — full login using useAutoForm)
    ├── DashboardPage.tsx            (NEW — welcome banner + stat cards)
    └── AdminsPage.tsx               (NEW — admins list placeholder)
```

## Navigation Config (`nav.ts`)

Single source of truth for all sidebar navigation.

```ts
import { LayoutDashboard, Users, type LucideIcon } from "lucide-react";

export interface NavItem {
  label: string;
  icon: LucideIcon;
  path?: string;              // direct link (no children)
  permissions?: string[];     // OR logic — any match grants access
  notificationKey?: string;   // key into notification store
  children?: NavChild[];      // expandable sub-menu
}

export interface NavChild {
  label: string;
  path: string;
  permissions?: string[];
  notificationKey?: string;
}

export const navigation: NavItem[] = [
  { label: "Dashboard", icon: LayoutDashboard, path: "/" },
  {
    label: "Admins",
    icon: Users,
    path: "/admins",
    permissions: ["admin.read", "admin.manage"],
  },
];
```

**Adding a nav item**: import icon, add one object to the array. Children auto-indent.

## Permission Filtering

```ts
function hasAccess(scopes: string[], required?: string[]): boolean {
  if (!required || required.length === 0) return true;
  return required.some((p) => scopes.includes(p));
}
```

- `permissions` omitted or empty → visible to all authenticated admins
- `permissions` set → admin must have at least one matching scope (matches backend `PermissionMode::Any`)
- Scopes come from `useAuthStore` → `account.scopes: string[]`

## Notification Badges

### Store stub (`stores/notifications.ts`)

```ts
interface NotificationState {
  counts: Record<string, number>;
  getCount: (key: string) => number;
}
```

Empty counts initially. Team implements polling/websocket later.

### Badge rendering rules

- **Leaf item** (no children): show `getCount(notificationKey)` if > 0
- **Parent with children**: auto-sum visible children's counts (permission-filtered). No notificationKey needed on parent.
- **Parent without children**: show own `getCount(notificationKey)` if > 0
- Badge style: `bg-primary` pill, white text, right-aligned

## Login Page

Uses `useAutoForm` with the existing admin auth store:

- Fields: `username` (text, required) + `password` (password, required)
- Centered card layout on `bg-background`
- Card: `bg-surface` with `border-border`, rounded
- Logo/brand area above fields
- General error banner above form
- Submit button: `bg-primary`, full width, loading spinner when busy
- On success: `setToken()` → `fetchAccount()` → navigate to `/`

## Admin Layout (Post-Login)

```
┌──────────────────────────────────────────────────────┐
│  Header (h-14, bg-surface, border-b border-border)   │
│  [☰ toggle]                          [name] [logout] │
├──────┬───────────────────────────────────────────────┤
│ Side │  Content Area (bg-background, p-6)            │
│ bar  │  <Outlet /> renders active page               │
│ w-64 │                                                │
│  or  │                                                │
│ w-16 │                                                │
│      │                                                │
│ [📊] │                                                │
│ [👥] │                                                │
└──────┴───────────────────────────────────────────────┘
```

- Sidebar: fixed left, below header, transitions width
- Header: fixed top, full width, z-10
- Content: margin-left matches sidebar width, margin-top = header height, scrollable

## Dashboard Page

- Welcome: "Welcome back, {account.name}"
- 4 stat cards in `grid-cols-1 sm:grid-cols-2 lg:grid-cols-4`
- Each card: icon, label, placeholder value, `bg-surface border border-border rounded-xl p-5`
- Stats: "Total Admins", "Active Today", "New This Week", "System Health"

## Admins Page

- Heading: "Admins"
- Subtitle: "Manage administrator accounts"
- Empty state placeholder directing team to wire up the datatable API

## CSS Additions (`app.css`)

New component classes added to existing `@layer components`:

```css
.rf-sidebar { ... }
.rf-sidebar-expanded { width: 16rem; }
.rf-sidebar-collapsed { width: 4rem; }
.rf-sidebar-link { ... }
.rf-sidebar-link-active { ... }
.rf-header { ... }
.rf-stat-card { ... }
```

All using existing theme tokens — no new CSS custom properties.

## Route Structure (`App.tsx`)

```tsx
<Routes>
  <Route path="/login" element={<LoginPage />} />
  <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
    <Route element={<AdminLayout />}>
      <Route index element={<DashboardPage />} />
      <Route path="/admins" element={<AdminsPage />} />
    </Route>
  </Route>
</Routes>
```

## Dependencies

Add to `frontend/package.json`:

```json
"lucide-react": "^0.468.0"
```
