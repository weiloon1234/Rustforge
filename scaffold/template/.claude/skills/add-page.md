---
name: add-page
description: Add a new frontend page to a portal
---

# Add a New Frontend Page

Follow these steps to add a new page to the admin or user portal.

## Step 1: Determine the portal

Decide which portal the page belongs to:
- `admin` -- for back-office / administrative pages
- `user` -- for customer-facing pages

## Step 2: Create the page component

Create `frontend/src/{portal}/pages/{group}/{Name}Page.tsx`:

```tsx
import { useTranslation } from "react-i18next";

export default function MyFeaturePage() {
  const { t } = useTranslation();

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">{t("My Feature")}</h1>
      <div>
        {/* Page content */}
      </div>
    </div>
  );
}
```

Conventions:
- Use default export for page components.
- Always use `useTranslation()` for all user-visible text -- never hardcode strings.
- Import shared components from `@shared/components` (e.g., `DataTable`, `Modal`, `Button`).
- Import portal-specific types from `@{portal}/types` (e.g., `@admin/types/MyType`).
- Use canonical Tailwind CSS classes (never arbitrary values when a built-in utility exists).
- Group pages by domain in subdirectories under `pages/`.

For pages that fetch data:
```tsx
import { useQuery } from "@tanstack/react-query";
import { api } from "@admin/api";  // or "@user/api" for user portal
import type { MyDomainOutput } from "@admin/types/MyDomainOutput";
import { useTranslation } from "react-i18next";

export default function MyFeaturePage() {
  const { t } = useTranslation();

  const { data, isLoading } = useQuery({
    queryKey: ["my-domain"],
    queryFn: () => api.get<MyDomainOutput[]>("/api/v1/admin/my-domain"),
  });

  if (isLoading) return <div>{t("Loading...")}</div>;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">{t("My Feature")}</h1>
      {/* Render data */}
    </div>
  );
}
```

Conventions for API imports:
- **Admin portal**: `import { api } from "@admin/api";`
- **User portal**: `import { api } from "@user/api";`
- Do NOT use `@shared/lib/api` -- each portal has its own API client with the correct base URL and auth headers.

## Step 3: Add the route

Update `frontend/src/{portal}/App.tsx`:

```tsx
import MyFeaturePage from "@{portal}/pages/{group}/MyFeaturePage";

// Inside the <Routes> block:
<Route path="/{group}/my-feature" element={<MyFeaturePage />} />
```

Conventions:
- Use kebab-case for URL paths.
- Lazy-load pages if they are large (use `React.lazy()` and `Suspense`).

## Step 4: Add sidebar navigation

Update `frontend/src/{portal}/nav.ts`:

```ts
{
  label: "My Feature",
  path: "/{group}/my-feature",
  icon: IconName,
  permission: "my_domain.read",  // Optional: gates visibility by permission
}
```

Add the entry in the appropriate nav group. The `permission` field controls whether the nav item is visible based on the user's permissions.

## Step 5: Add i18n translations

Edit `i18n/en.json`: Add entries only where the key differs from the value.

Edit `i18n/zh.json`: Always add Chinese translations for:
- Page title
- All labels, buttons, and messages used on the page

```json
{
  "My Feature": "我的功能",
  "Some Button Label": "某按钮标签"
}
```

## Step 6: Verify

```bash
npm --prefix frontend run typecheck
```

Common issues:
- Missing type imports -- ensure `make gen-types` has been run if new backend types are needed.
- Missing i18n keys -- causes untranslated text in non-English locales.
- Incorrect path alias -- use `@admin/`, `@user/`, or `@shared/` (defined in `tsconfig.json`).
- Using `@shared/lib/api` instead of the portal-specific `@{portal}/api`.
