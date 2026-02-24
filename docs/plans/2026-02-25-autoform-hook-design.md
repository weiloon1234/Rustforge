# useAutoForm Hook Design

## Problem

Building forms in the scaffolded React app requires manually wiring state, change handlers, error display, and API submission for every form. This is repetitive and error-prone. We need a single hook that auto-generates form UI from a field definition, handles submission, and auto-maps backend validation errors to the correct fields.

## Solution

A `useAutoForm` hook in `frontend/src/shared/useAutoForm.tsx` that:

1. Accepts the portal's Axios instance and a typed field config
2. Returns `{ submit, busy, form, errors, reset, setValues }`
3. Renders form fields using the existing shared components (TextInput, TextArea, Select, Checkbox, Radio)
4. Auto-maps `ApiErrorResponse.errors` to per-field error messages
5. Uses a 2-column CSS grid with per-field `span` control

## API

### Signature

```ts
function useAutoForm(api: AxiosInstance, config: AutoFormConfig): AutoFormResult
```

### AutoFormConfig

```ts
interface AutoFormConfig {
  url: string;
  method?: "post" | "put" | "patch";  // default: "post"
  fields: FieldDef[];
  defaults?: Record<string, string>;
  onSuccess?: (data: unknown) => void;
  onError?: (error: unknown) => void;
}
```

### FieldDef (discriminated union)

```ts
type FieldDef =
  | { name: string; type: "text" | "email" | "password" | "search" | "url" | "tel" | "number" | "money" | "pin"; label: string; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean }
  | { name: string; type: "textarea"; label: string; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean; rows?: number }
  | { name: string; type: "select"; label: string; options: { value: string; label: string; disabled?: boolean }[]; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean }
  | { name: string; type: "checkbox"; label: string; span?: 1 | 2; required?: boolean; notes?: string; disabled?: boolean }
  | { name: string; type: "radio"; label: string; options: { value: string; label: string; disabled?: boolean }[]; span?: 1 | 2; required?: boolean; notes?: string; disabled?: boolean };
```

- `span` defaults to `2` (full width). `span: 1` = half width (2 fields per row).
- All types except checkbox/radio/select map to TextInput with the corresponding `type` prop.

### AutoFormResult

```ts
interface AutoFormResult {
  submit: () => Promise<void>;
  busy: boolean;
  form: React.ReactElement;
  errors: {
    general: string | null;
    fields: Record<string, string>;
  };
  reset: () => void;
  setValues: (values: Record<string, string>) => void;
}
```

## Internal State

- `values: Record<string, string>` — initialized from `defaults`, updated on change
- `fieldErrors: Record<string, string>` — per-field first error from backend
- `generalError: string | null` — top-level message from backend
- `busy: boolean` — true during API call

## Data Flow

1. Hook creates internal state for values, errors, busy
2. `form` renders a `<div className="rf-form-grid">` (2-column CSS grid)
3. Each field renders the matching component with auto-wired value/onChange/error
4. Field `span` controls `gridColumn: span 1` or `span 2`
5. `submit()` → sets busy, clears errors, calls `api[method](url, values)`
6. On success (2xx) → calls `onSuccess(response.data.data)`
7. On error (422/400) → parses `ApiErrorResponse`, maps `errors` to fields, sets `general`
8. Field errors auto-clear when user changes that field

## CSS Addition

Each portal's `app.css` needs one new class:

```css
.rf-form-grid {
  @apply grid grid-cols-2 gap-x-4;
}
```

## Files

- New: `frontend/src/shared/useAutoForm.tsx`
- Modified: `frontend/src/shared/components/index.ts` (re-export)
- Modified: `frontend/src/admin/app.css` (add rf-form-grid)
- Modified: `frontend/src/user/app.css` (add rf-form-grid)
- Modified: `scaffold/src/templates.rs` (add template for useAutoForm, update index.ts and app.css templates)

## Usage Example

```tsx
import { useAutoForm } from "../shared/useAutoForm";
import { api } from "../api";

function CreateAdminPage() {
  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "/api/v1/admin",
    fields: [
      { name: "name", type: "text", label: "Name", span: 2, required: true },
      { name: "email", type: "email", label: "Email", span: 1, required: true },
      { name: "username", type: "text", label: "Username", span: 1, required: true },
      { name: "password", type: "password", label: "Password", span: 1, required: true },
      { name: "password_confirmation", type: "password", label: "Confirm", span: 1, required: true },
      { name: "role", type: "select", label: "Role", span: 2, options: [
        { value: "admin", label: "Admin" },
        { value: "editor", label: "Editor" },
      ]},
      { name: "bio", type: "textarea", label: "Bio", span: 2, rows: 4 },
      { name: "active", type: "checkbox", label: "Active", span: 2 },
    ],
    onSuccess: () => navigate("/admins"),
  });

  return (
    <div>
      <h1>Create Admin</h1>
      {errors.general && <p className="rf-error-message">{errors.general}</p>}
      {form}
      <button onClick={submit} disabled={busy}>
        {busy ? "Creating..." : "Create"}
      </button>
    </div>
  );
}
```
