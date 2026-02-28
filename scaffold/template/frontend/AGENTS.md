# Frontend — Multi-Portal React + Vite + Tailwind 4

This directory contains the frontend source for the Rustforge starter. It ships two independent SPA portals:

| Portal | Base | Dev port | Build output |
|--------|------|----------|--------------|
| **user** | `/` | 5173 | `../public/` (root) |
| **admin** | `/admin/` | 5174 | `../public/admin/` |

Each portal has its own Vite config, HTML entry, CSS theme, and source tree.

## Directory Structure

```
frontend/
├── package.json
├── postcss.config.js
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.user.ts
├── vite.config.admin.ts
├── user.html
├── admin.html
└── src/
    ├── shared/                        # Cross-portal code
    │   ├── types/                     # Generated shared TS types (make gen-types)
    │   │   ├── api.ts                 # ApiResponse<T>, ApiErrorResponse
    │   │   ├── datatable.ts           # DataTable request/response generics
    │   │   ├── platform.ts            # Localized, attachments, meta, JSON generic shapes
    │   │   └── index.ts               # Barrel export
    │   ├── i18n.ts                    # i18next init (shared JSON, :param transform)
    │   ├── createAuthStore.ts         # Zustand auth store factory
    │   ├── createApiClient.ts         # Axios factory with interceptors
    │   ├── ProtectedRoute.tsx         # Auth guard (route protection + session restore)
    │   └── components/                # Shared form components (styled via rf-* classes)
    │       ├── index.ts               # Barrel export
    │       ├── FieldErrors.tsx          # Shared error renderer (FieldErrors, hasFieldError)
    │       ├── TextInput.tsx           # text, email, password, search, url, tel, number, money, pin
    │       ├── TextArea.tsx            # Multi-line text
    │       ├── Select.tsx              # Dropdown with typed options
    │       ├── Checkbox.tsx            # Single checkbox
    │       └── Radio.tsx               # Radio group with typed options
    ├── user/
    │   ├── main.tsx                   # Entry (BrowserRouter)
    │   ├── App.tsx                    # Routes
    │   ├── app.css                    # Tailwind 4 theme
    │   ├── api.ts                     # Axios instance for this portal
    │   ├── stores/auth.ts             # Auth store instance
    │   └── types/                     # Generated user TS types (make gen-types)
    │       └── index.ts               # Barrel export (expand as user contracts are added)
    └── admin/
        ├── main.tsx                   # Entry (BrowserRouter basename="/admin")
        ├── App.tsx                    # Routes
        ├── app.css                    # Tailwind 4 theme
        ├── api.ts                     # Axios instance for this portal
        ├── stores/auth.ts             # Auth store instance
        └── types/                     # Generated admin TS types (make gen-types)
            ├── enums.ts               # AdminType, Permission, AuthClientType
            ├── admin.ts               # CRUD: CreateAdminInput, AdminOutput, etc.
            ├── admin-auth.ts          # Auth: AdminLoginInput, AdminMeOutput, etc.
            ├── datatable-admin.ts     # AdminDatatableQueryInput, etc.
            └── index.ts               # Barrel export
```

## Commands

```bash
make dev              # All: Vite user + Vite admin + Rust API
make dev-user         # Vite user portal only (port 5173)
make dev-admin        # Vite admin portal only (port 5174)
make dev-api          # Rust API only (cargo-watch, port 3000)
make build-frontend   # Clean build all portals → public/
```

## Routing (React Router)

Each portal uses `BrowserRouter` from `react-router-dom`. The admin portal sets `basename="/admin"` so all routes are relative to `/admin/`.

Use `<Link to="/login">` and `useNavigate()` — the basename is applied automatically.

### Protected Routes (Auth Guard)

`ProtectedRoute` in `shared/ProtectedRoute.tsx` is the auth middleware. Wrap any routes that require authentication:

```tsx
import { Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "@shared/ProtectedRoute";
import { useAuthStore } from "@user/stores/auth";

export default function App() {
  return (
    <Routes>
      {/* Public routes */}
      <Route path="/login" element={<LoginPage />} />

      {/* Protected routes — redirect to /login if unauthenticated */}
      <Route element={<ProtectedRoute useAuthStore={useAuthStore} />}>
        <Route path="/*" element={<DashboardPage />} />
      </Route>
    </Routes>
  );
}
```

What `ProtectedRoute` does on mount:
1. Calls `initSession()` — checks if a persisted token exists
2. If token exists → calls `fetchAccount()` to validate it
3. If access token expired → calls `refreshToken()` (browser sends HttpOnly cookie) → retries `fetchAccount()`
4. If refresh also fails → clears auth state
5. Shows a loading screen while initializing
6. Once initialized, redirects to `/login` if no valid token, otherwise renders child routes via `<Outlet />`

The `from` location is passed in the redirect state, so after login you can navigate back:

```tsx
const location = useLocation();
const from = location.state?.from?.pathname || "/";
// After successful login:
navigate(from, { replace: true });
```

### Custom login path

Pass `loginPath` prop if the portal uses a different login route:

```tsx
<Route element={<ProtectedRoute useAuthStore={useAuthStore} loginPath="/auth/signin" />}>
```

## API Client (Axios)

Each portal has its own `api.ts` that exports a configured Axios instance. The shared factory (`createApiClient`) provides:

- **Request interceptor**: attaches `Authorization: Bearer <token>` from the auth store
- **Response interceptor**: on 401, attempts token refresh (one concurrent refresh), retries the request, or redirects to login on failure

```tsx
// Import the portal's api instance for all API calls (use @admin/ or @user/ alias)
import { api } from "@admin/api";

const res = await api.get("/api/v1/articles");
const data = res.data;
```

The refresh uses `client_type: "web"` — the Rust backend stores the refresh token in an HttpOnly cookie. The frontend only manages the access token; the browser sends the cookie automatically.

### Auth Flow

1. **Login**: `useAuthStore.login({ username, password })` → POST with `client_type: "web"` → stores `access_token`, refresh token set as HttpOnly cookie by server
2. **Page refresh**: `ProtectedRoute` calls `initSession()` → validates persisted token → refreshes if expired → loads account data
3. **API calls**: Axios attaches bearer token automatically
4. **401 response**: interceptor calls `refreshToken()` → POST to refresh endpoint (cookie sent automatically) → new `access_token` → retry original request
5. **Refresh failure**: clears auth state, redirects to `/login`

## i18n (Shared with Rust)

Frontend and Rust share the same `i18n/*.json` files. The Rust backend uses `:param` syntax; `src/shared/i18n.ts` transforms `:param` → `{{param}}` at init time so i18next can interpolate.

```tsx
import { useTranslation } from "react-i18next";

function Greeting({ name }: { name: string }) {
  const { t } = useTranslation();
  return <p>{t("Welcome :name", { name })}</p>;
}
```

The key is the English text itself — if no translation is found, the key is the fallback.

## TypeScript Types (Generated)

Type definitions in `*/types/` directories are **auto-generated** from Rust contract structs using `ts-rs`. Do not edit them manually — run `make gen-types` to regenerate after changing Rust contracts.

### Usage

```typescript
import type { ApiResponse } from "@shared/types";
import type { AdminOutput, CreateAdminInput } from "@admin/types";

// Typed API calls
const res = await api.post<ApiResponse<AdminOutput>>("/api/v1/admin", input);
const admin: AdminOutput = res.data.data;
```

### Regeneration

```bash
make gen-types    # Regenerate frontend TS types from Rust contracts
make gen          # Code generation + type generation
```

### How it works

1. Rust contract structs derive `ts_rs::TS` with `#[ts(export, export_to = "{portal}/types/")]`
2. `app/build.rs` auto-discovers contract/datatable TS types from `app/src/contracts/api/v1/**` and `app/src/contracts/datatable/**`
3. `app/src/bin/export-types.rs` exports discovered types, assembles `.ts` files with enum imports, and writes to `frontend/src/`
4. Per-portal `types/index.ts`, shared framework types, and enum types are emitted automatically

### Adding types for a new domain

1. In your Rust contract, add `#[derive(TS)]` and `#[ts(export, export_to = "{portal}/types/")]`
2. For fields using external types (generated enums, framework types), add `#[ts(type = "TypeName")]`
3. Run `make gen-types` (types and portal barrels are discovered/generated automatically)

### Type mapping conventions

| Rust | TypeScript | Notes |
|------|-----------|-------|
| `String` | `string` | |
| `i64`, `f64` | `number` | |
| `bool` | `boolean` | |
| `Option<T>` | `T \| null` | |
| `Vec<T>` | `T[]` | |
| `time::OffsetDateTime` | `string` | Use `#[ts(type = "string")]` |
| `UsernameString` (newtype) | `string` | Use `#[ts(type = "string")]` |
| `AdminType` (generated enum) | `AdminType` | Use `#[ts(type = "AdminType")]` |
| `#[serde(skip)]` field | omitted | ts-rs respects serde attrs |

## State Management (Zustand)

Use Zustand for state. Define stores in `src/{portal}/stores/`.

### Auth Store Factory

`src/shared/createAuthStore.ts` is a factory that creates a typed auth store for any portal. Each portal provides its own endpoints:

```typescript
// src/{portal}/stores/auth.ts
import { createAuthStore } from "@shared/createAuthStore";

export const useAuthStore = createAuthStore({
  loginEndpoint:   "/api/v1/{portal}/auth/login",
  meEndpoint:      "/api/v1/{portal}/auth/me",
  refreshEndpoint: "/api/v1/{portal}/auth/refresh",
  storageKey:      "{portal}-auth",
});
```

The `login` action accepts a generic credentials object — each portal passes whatever fields its API expects:

```tsx
// Admin login (uses username)
await login({ username, password });

// User login (might use email)
await login({ email, password });
```

`client_type: "web"` is appended automatically.

For portals with extra account fields, pass a generic:

```typescript
import { createAuthStore, type Account } from "@shared/createAuthStore";

interface MerchantAccount extends Account {
  companyId: number;
  companyName: string;
}

export const useAuthStore = createAuthStore<MerchantAccount>({
  loginEndpoint:   "/api/v1/merchant/auth/login",
  meEndpoint:      "/api/v1/merchant/auth/me",
  refreshEndpoint: "/api/v1/merchant/auth/refresh",
  storageKey:      "merchant-auth",
});
```

### Creating Other Shared Store Factories

Follow the same factory pattern as `createAuthStore` for any cross-portal store. Define the factory in `shared/`, instantiate with portal-specific config in `src/{portal}/stores/`.

## Tailwind CSS 4

Each portal customises its design tokens in its own `app.css` via `@theme { }`. No `tailwind.config.js` is used — Tailwind 4 reads theme configuration from CSS.

```css
@import "tailwindcss";

@theme {
  --color-primary: #2563eb;
}
```

### Theme Tokens

Each portal defines a comprehensive set of semantic color tokens in `@theme`. The admin portal uses a dark scheme and the user portal uses a light scheme. Key token groups:

| Group | Tokens | Purpose |
|-------|--------|---------|
| **Base** | `background`, `foreground`, `muted`, `muted-foreground` | Page background, text, subtle text |
| **Surface** | `surface`, `surface-hover`, `surface-active` | Cards, panels, interactive elements |
| **Primary** | `primary`, `primary-hover`, `primary-foreground` | Brand color, buttons, links |
| **Border** | `border`, `border-hover` | General dividers, card borders |
| **Input** | `input`, `input-border`, `input-border-hover`, `input-focus`, `input-placeholder`, `input-disabled` | Form control styling |
| **Ring** | `ring` | Focus ring color |
| **Status** | `error`/`error-muted`, `warning`/`warning-muted`, `success`/`success-muted`, `info`/`info-muted` | Validation, alerts, badges |

## Shared Form Components

Reusable form components live in `src/shared/components/`. They contain **zero hardcoded Tailwind utilities** — all visual styling is applied through `rf-*` CSS classes defined in each portal's `app.css` using `@layer components` + `@apply`.

This means portals can have completely different visual styles while sharing identical React logic.

### Available Components

| Component | Import | Description |
|-----------|--------|-------------|
| `TextInput` | `TextInputProps` | Text, email, password, search, url, tel, number + special `money` and `pin` types |
| `TextArea` | `TextAreaProps` | Multi-line text input |
| `Select` | `SelectProps`, `SelectOption` | Dropdown with typed options |
| `Checkbox` | `CheckboxProps` | Single checkbox with label |
| `Radio` | `RadioProps`, `RadioOption` | Radio group with typed options |

### Usage

```tsx
import { TextInput, TextArea, Select, Checkbox, Radio } from "@shared/components";

// Basic text input with error
<TextInput label="Email" type="email" required error={errors.email} />

// Money input — displays formatted (1,234.56), onChange emits raw numeric string
<TextInput label="Amount" type="money" onChange={(e) => setAmount(e.target.value)} />

// PIN input — renders as password field, strips non-digits, numeric keyboard
<TextInput label="PIN" type="pin" maxLength={6} />

// Text area with helper notes
<TextArea label="Bio" notes="Maximum 500 characters" rows={4} />

// Select with placeholder
<Select
  label="Country"
  placeholder="Choose a country..."
  options={[
    { value: "us", label: "United States" },
    { value: "uk", label: "United Kingdom" },
  ]}
  required
/>

// Checkbox
<Checkbox label="I agree to the terms" error={errors.terms} />

// Radio group
<Radio
  name="role"
  label="Role"
  value={role}
  onChange={setRole}
  options={[
    { value: "admin", label: "Administrator" },
    { value: "editor", label: "Editor" },
    { value: "viewer", label: "Viewer" },
  ]}
/>
```

### Error and Notes Pattern

All components follow the same pattern:
- `error?: string` prop: shows a single red error message below the input (for standalone usage)
- `errors?: string[]` prop: shows multiple red error messages, one per line (for API validation errors)
- Both can be provided simultaneously — duplicates are automatically deduplicated by `FieldErrors`
- `notes` prop: shows a grey helper note below the input (hidden when any error is present)
- `required` prop: adds a red asterisk after the label

`useAutoForm` passes `errors` (array) from the API response directly to each component, preserving individual validation messages.

### Special TextInput Types

- **`money`**: Formats display value with commas (`1,234.56`), emits raw numeric string via `onChange`, uses `inputMode="decimal"` for mobile numeric keyboard
- **`pin`**: Renders as `type="password"`, strips non-digit characters, uses `inputMode="numeric"` for mobile numeric keyboard

### CSS Class Reference

Each portal's `app.css` defines these `rf-*` classes using `@apply` with theme tokens:

| Class | Used by | Purpose |
|-------|---------|---------|
| `rf-field` | All | Wrapper div with bottom margin |
| `rf-label` | All | Label styling |
| `rf-label-required` | All | Adds red asterisk via `::after` |
| `rf-input` / `rf-input-error` | TextInput | Text input styling |
| `rf-textarea` / `rf-textarea-error` | TextArea | Textarea styling |
| `rf-select` / `rf-select-error` / `rf-select-placeholder` | Select | Select dropdown styling |
| `rf-checkbox-wrapper` / `rf-checkbox` / `rf-checkbox-error` / `rf-checkbox-label` | Checkbox | Checkbox layout and styling |
| `rf-radio-group` / `rf-radio-wrapper` / `rf-radio` / `rf-radio-error` / `rf-radio-label` | Radio | Radio group layout and styling |
| `rf-error-message` | All | Error text below input |
| `rf-note` | All | Helper text below input |

### Theming for New Portals

When adding a new portal, copy the `@layer components` block from an existing portal's `app.css`. The visual appearance is controlled entirely by the `@theme` tokens — the same `rf-*` class definitions produce different results based on each portal's token values.

## Adding a New Portal

Use the admin portal as the reference. Example below uses `merchant` on port 5175.

### 1. Vite config — `frontend/vite.config.merchant.ts`

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  root: ".",
  base: "/merchant/",
  build: {
    outDir: "../public/merchant",
    emptyOutDir: true,
    rollupOptions: { input: "merchant.html" },
  },
  experimental: {
    renderBuiltUrl(filename, { hostType }) {
      if (hostType === "html") return filename;
      return "/merchant/" + filename;
    },
  },
  server: {
    port: 5175,
    proxy: { "/api": "http://localhost:3000" },
  },
});
```

Key settings: `base: "/merchant/"` (trailing slash), `outDir: "../public/merchant"`, unique `port`.

### 2. HTML entry — `frontend/merchant.html`

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Merchant</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/merchant/main.tsx"></script>
  </body>
</html>
```

### 3. Source directory — `frontend/src/merchant/`

```
src/merchant/
├── main.tsx          # BrowserRouter with basename="/merchant"
├── App.tsx           # Routes + ProtectedRoute
├── app.css           # @import "tailwindcss" + @theme {} + rf-* classes
├── api.ts            # createApiClient wired to auth store
├── stores/
│   └── auth.ts       # createAuthStore with /api/v1/merchant/auth/* endpoints
└── types/            # Generated TS types (make gen-types)
    └── index.ts
```

**`main.tsx`** — must set `basename`:

```tsx
import { BrowserRouter } from "react-router-dom";
// ...
<BrowserRouter basename="/merchant">
  <App />
</BrowserRouter>
```

**`app.css`** — copy the `@theme {}` block and `@layer components` block from an existing portal, then customise the colour tokens. The `rf-*` class definitions in `@layer components` should be identical — visual differences come from the theme tokens.

### 4. npm scripts — `frontend/package.json`

```json
"dev:merchant": "vite --config vite.config.merchant.ts",
"build:merchant": "vite build --config vite.config.merchant.ts",
"build": "rm -rf ../public && npm run build:admin && npm run build:merchant && npm run build:user"
```

Build order: nested portals (`admin`, `merchant`) **before** `user`. The user build uses `emptyOutDir: false` so it preserves the nested portal outputs inside `public/`.

### 5. Makefile

Add a `dev-merchant` target and include the new process in `dev`:

```makefile
.PHONY: dev-merchant
dev-merchant: ensure-frontend-deps
	npm --prefix frontend run dev:merchant

# In the `dev` target, add a line:
npm --prefix frontend run dev:merchant &
```

### 6. Rust — SPA serving (`app/src/internal/api/mod.rs`)

Add **before** the user SPA catch-all block. Two modes:

**Production** (built frontend exists): serve static files with SPA fallback.
**Dev** (no built frontend): serve HTML that loads from the Vite dev server with HMR.

```rust
// Merchant SPA: /merchant/* → public/merchant/index.html
let merchant_public = public_path.join("merchant");
let merchant_index = merchant_public.join("index.html");
if merchant_public.is_dir() && merchant_index.is_file() {
    router = router.nest_service(
        "/merchant",
        ServeDir::new(&merchant_public).fallback(ServeFile::new(&merchant_index)),
    );
} else {
    router = router
        .route("/merchant", axum_get(merchant_dev))
        .route("/merchant/{*path}", axum_get(merchant_dev));
}
```

Dev handler — serves HTML that loads scripts from the Vite dev server so HMR and React Fast Refresh work at `localhost:3000/merchant`:

```rust
async fn merchant_dev() -> Html<&'static str> {
    Html(r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Merchant</title>
    <script type="module" src="http://localhost:5175/@vite/client"></script>
    <script type="module">
      import RefreshRuntime from "http://localhost:5175/@react-refresh"
      RefreshRuntime.injectIntoGlobalHook(window)
      window.$RefreshReg$ = () => {}
      window.$RefreshSig$ = () => (type) => type
      window.__vite_plugin_react_preamble_installed__ = true
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="http://localhost:5175/src/merchant/main.tsx"></script>
  </body>
</html>"#)
}
```

### Port allocation

| Portal | Dev port | Base path |
|--------|----------|-----------|
| user | 5173 | `/` |
| admin | 5174 | `/admin/` |
| (next) | 5175 | `/{name}/` |

## Dev vs Production Serving

**Dev mode** (`make dev`, no built frontend in `public/`):

The Rust API server at `:3000` serves HTML pages that load scripts directly from the Vite dev servers. The browser fetches modules from the Vite origin, so HMR, React Fast Refresh, and all asset resolution work as if you visited the Vite port directly. You can visit either `localhost:3000` or the Vite port — both work.

**Production** (`make build-frontend`):

`make build-frontend` compiles all portals into `public/`. The Rust API serves them as static files with SPA fallback routing:

- `/admin/*` → `public/admin/index.html`
- `/{portal}/*` → `public/{portal}/index.html`
- `/*` → `public/index.html` (user portal catch-all, must be last)
