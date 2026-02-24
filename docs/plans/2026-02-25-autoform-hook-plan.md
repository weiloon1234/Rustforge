# useAutoForm Hook Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `useAutoForm` hook to the scaffold that auto-builds forms from field definitions, with submit/busy/errors/reset and backend validation error auto-mapping.

**Architecture:** A single `useAutoForm.tsx` file in `frontend/src/shared/` exports a hook that manages form state internally and renders a 2-column CSS grid of the existing shared components (TextInput, TextArea, Select, Checkbox, Radio). The scaffold's `templates.rs` and `main.rs` are updated to include this new file in generated projects.

**Tech Stack:** React 19, TypeScript, Axios, existing rf-* CSS class system.

---

### Task 1: Create useAutoForm.tsx in the generated project

**Files:**
- Create: `frontend/src/shared/useAutoForm.tsx` (in `/private/tmp/rustforge-test-scaffold/`)

**Step 1: Create the useAutoForm.tsx file**

Write the full hook implementation to `/private/tmp/rustforge-test-scaffold/frontend/src/shared/useAutoForm.tsx`:

```tsx
import { useState, useMemo, useCallback, type ReactElement } from "react";
import type { AxiosInstance, AxiosError } from "axios";
import { TextInput } from "./components/TextInput";
import { TextArea } from "./components/TextArea";
import { Select, type SelectOption } from "./components/Select";
import { Checkbox } from "./components/Checkbox";
import { Radio, type RadioOption } from "./components/Radio";

type InputFieldType = "text" | "email" | "password" | "search" | "url" | "tel" | "number" | "money" | "pin";

type FieldDef =
  | { name: string; type: InputFieldType; label: string; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean }
  | { name: string; type: "textarea"; label: string; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean; rows?: number }
  | { name: string; type: "select"; label: string; options: SelectOption[]; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean }
  | { name: string; type: "checkbox"; label: string; span?: 1 | 2; required?: boolean; notes?: string; disabled?: boolean }
  | { name: string; type: "radio"; label: string; options: RadioOption[]; span?: 1 | 2; required?: boolean; notes?: string; disabled?: boolean };

interface AutoFormConfig {
  url: string;
  method?: "post" | "put" | "patch";
  fields: FieldDef[];
  defaults?: Record<string, string>;
  onSuccess?: (data: unknown) => void;
  onError?: (error: unknown) => void;
}

interface AutoFormErrors {
  general: string | null;
  fields: Record<string, string>;
}

interface AutoFormResult {
  submit: () => Promise<void>;
  busy: boolean;
  form: ReactElement;
  errors: AutoFormErrors;
  reset: () => void;
  setValues: (values: Record<string, string>) => void;
}

export type { FieldDef, AutoFormConfig, AutoFormErrors, AutoFormResult };

function buildDefaults(fields: FieldDef[], defaults?: Record<string, string>): Record<string, string> {
  const values: Record<string, string> = {};
  for (const field of fields) {
    values[field.name] = defaults?.[field.name] ?? "";
  }
  return values;
}

export function useAutoForm(api: AxiosInstance, config: AutoFormConfig): AutoFormResult {
  const { url, method = "post", fields, defaults, onSuccess, onError } = config;

  const [values, setValuesState] = useState<Record<string, string>>(() => buildDefaults(fields, defaults));
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [generalError, setGeneralError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const setValue = useCallback((name: string, value: string) => {
    setValuesState((prev) => ({ ...prev, [name]: value }));
    setFieldErrors((prev) => {
      if (!prev[name]) return prev;
      const next = { ...prev };
      delete next[name];
      return next;
    });
  }, []);

  const reset = useCallback(() => {
    setValuesState(buildDefaults(fields, defaults));
    setFieldErrors({});
    setGeneralError(null);
  }, [fields, defaults]);

  const setValues = useCallback((incoming: Record<string, string>) => {
    setValuesState((prev) => ({ ...prev, ...incoming }));
  }, []);

  const submit = useCallback(async () => {
    setBusy(true);
    setFieldErrors({});
    setGeneralError(null);

    // Build payload — checkboxes send "1"/"0" instead of "on"/""
    const payload: Record<string, string> = {};
    for (const field of fields) {
      const v = values[field.name] ?? "";
      payload[field.name] = field.type === "checkbox" ? (v ? "1" : "0") : v;
    }

    try {
      const response = await api[method](url, payload);
      onSuccess?.(response.data?.data ?? response.data);
    } catch (err) {
      const axiosErr = err as AxiosError<{ message?: string; errors?: Record<string, string[]> }>;
      const body = axiosErr.response?.data;
      if (body) {
        setGeneralError(body.message ?? "Something went wrong");
        if (body.errors) {
          const mapped: Record<string, string> = {};
          for (const [key, msgs] of Object.entries(body.errors)) {
            if (msgs.length > 0) mapped[key] = msgs[0];
          }
          setFieldErrors(mapped);
        }
      } else {
        setGeneralError("Something went wrong");
      }
      onError?.(err);
    } finally {
      setBusy(false);
    }
  }, [api, method, url, fields, values, onSuccess, onError]);

  const form = useMemo((): ReactElement => {
    return (
      <div className="rf-form-grid">
        {fields.map((field) => {
          const span = field.span ?? 2;
          const style = { gridColumn: `span ${span}` };
          const error = fieldErrors[field.name];

          switch (field.type) {
            case "textarea":
              return (
                <div key={field.name} style={style}>
                  <TextArea
                    label={field.label}
                    value={values[field.name] ?? ""}
                    onChange={(e) => setValue(field.name, e.target.value)}
                    error={error}
                    notes={field.notes}
                    placeholder={field.placeholder}
                    required={field.required}
                    disabled={field.disabled}
                    rows={field.rows}
                  />
                </div>
              );

            case "select":
              return (
                <div key={field.name} style={style}>
                  <Select
                    label={field.label}
                    options={field.options}
                    value={values[field.name] ?? ""}
                    onChange={(e) => setValue(field.name, e.target.value)}
                    error={error}
                    notes={field.notes}
                    placeholder={field.placeholder}
                    required={field.required}
                    disabled={field.disabled}
                  />
                </div>
              );

            case "checkbox":
              return (
                <div key={field.name} style={style}>
                  <Checkbox
                    label={field.label}
                    checked={values[field.name] === "1"}
                    onChange={(e) => setValue(field.name, e.target.checked ? "1" : "")}
                    error={error}
                    notes={field.notes}
                    required={field.required}
                    disabled={field.disabled}
                  />
                </div>
              );

            case "radio":
              return (
                <div key={field.name} style={style}>
                  <Radio
                    name={field.name}
                    label={field.label}
                    options={field.options}
                    value={values[field.name] ?? ""}
                    onChange={(v) => setValue(field.name, v)}
                    error={error}
                    notes={field.notes}
                    required={field.required}
                    disabled={field.disabled}
                  />
                </div>
              );

            default: {
              // All TextInput types: text, email, password, search, url, tel, number, money, pin
              const inputField = field as FieldDef & { type: InputFieldType };
              return (
                <div key={field.name} style={style}>
                  <TextInput
                    type={inputField.type}
                    label={field.label}
                    value={values[field.name] ?? ""}
                    onChange={(e) => setValue(field.name, e.target.value)}
                    error={error}
                    notes={field.notes}
                    placeholder={(field as { placeholder?: string }).placeholder}
                    required={field.required}
                    disabled={field.disabled}
                  />
                </div>
              );
            }
          }
        })}
      </div>
    );
  }, [fields, values, fieldErrors, setValue]);

  return { submit, busy, form, errors: { general: generalError, fields: fieldErrors }, reset, setValues };
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd /private/tmp/rustforge-test-scaffold/frontend && npx tsc --noEmit src/shared/useAutoForm.tsx 2>&1 | head -20`

> Note: If tsc is not installed or there are module resolution issues, just verify the file was created correctly by reading it. The real TypeScript verification happens after all templates are wired.

**Step 3: Commit**

```bash
git add frontend/src/shared/useAutoForm.tsx
git commit -m "feat: add useAutoForm hook — form builder with auto error mapping"
```

---

### Task 2: Update shared components index.ts to re-export useAutoForm

**Files:**
- Modify: `/private/tmp/rustforge-test-scaffold/frontend/src/shared/components/index.ts`

**Step 1: Add re-export**

Add these lines at the end of `frontend/src/shared/components/index.ts`:

```ts
export { useAutoForm } from "../useAutoForm";
export type { FieldDef, AutoFormConfig, AutoFormErrors, AutoFormResult } from "../useAutoForm";
```

**Step 2: Commit**

```bash
git add frontend/src/shared/components/index.ts
git commit -m "feat: re-export useAutoForm from shared components barrel"
```

---

### Task 3: Add rf-form-grid CSS class to both portal app.css files

**Files:**
- Modify: `/private/tmp/rustforge-test-scaffold/frontend/src/admin/app.css`
- Modify: `/private/tmp/rustforge-test-scaffold/frontend/src/user/app.css`

**Step 1: Add rf-form-grid to admin app.css**

Add inside the `@layer components { ... }` block, after the `.rf-note` line (line 95) and before the closing `}`:

```css
  .rf-form-grid { @apply grid grid-cols-2 gap-x-4; }
```

**Step 2: Add rf-form-grid to user app.css**

Same change — add inside `@layer components { ... }` block, after `.rf-note` line, before closing `}`:

```css
  .rf-form-grid { @apply grid grid-cols-2 gap-x-4; }
```

**Step 3: Commit**

```bash
git add frontend/src/admin/app.css frontend/src/user/app.css
git commit -m "feat: add rf-form-grid CSS class to both portal stylesheets"
```

---

### Task 4: Add useAutoForm template to scaffold/src/templates.rs

**Files:**
- Modify: `/Users/weiloonso/Projects/personal/Rust/Rustforge/scaffold/src/templates.rs`

**Step 1: Add the FRONTEND_SRC_SHARED_USE_AUTO_FORM_TSX constant**

Insert a new `pub const` after the existing `FRONTEND_SRC_SHARED_PROTECTED_ROUTE_TSX` constant (around line 5125, before `FRONTEND_SRC_SHARED_COMPONENTS_INDEX_TS`). The constant should contain the exact content of the `useAutoForm.tsx` file created in Task 1.

The constant name: `FRONTEND_SRC_SHARED_USE_AUTO_FORM_TSX`

Wrap in `r##"..."##` (double hash raw string) since the content contains `#` characters in import paths.

Copy the exact content from `/private/tmp/rustforge-test-scaffold/frontend/src/shared/useAutoForm.tsx`.

**Step 2: Update the FRONTEND_SRC_SHARED_COMPONENTS_INDEX_TS constant**

Find the existing `FRONTEND_SRC_SHARED_COMPONENTS_INDEX_TS` constant and append the useAutoForm re-exports:

```ts
export { useAutoForm } from "../useAutoForm";
export type { FieldDef, AutoFormConfig, AutoFormErrors, AutoFormResult } from "../useAutoForm";
```

**Step 3: Update FRONTEND_SRC_ADMIN_APP_CSS constant**

Find the `FRONTEND_SRC_ADMIN_APP_CSS` constant and add `.rf-form-grid { @apply grid grid-cols-2 gap-x-4; }` inside the `@layer components` block, after `.rf-note`.

**Step 4: Update FRONTEND_SRC_USER_APP_CSS constant**

Same change to the `FRONTEND_SRC_USER_APP_CSS` constant.

**Step 5: Commit**

```bash
git add scaffold/src/templates.rs
git commit -m "feat: add useAutoForm template + update index/css templates"
```

---

### Task 5: Register the new template in scaffold/src/main.rs

**Files:**
- Modify: `/Users/weiloonso/Projects/personal/Rust/Rustforge/scaffold/src/main.rs`

**Step 1: Add FileTemplate entry**

Find the block where `FRONTEND_SRC_SHARED_PROTECTED_ROUTE_TSX` is registered (around line 608-611). Insert a new `FileTemplate` entry **after** ProtectedRoute and **before** the components/index.ts entry:

```rust
        FileTemplate {
            path: "frontend/src/shared/useAutoForm.tsx",
            content: templates::FRONTEND_SRC_SHARED_USE_AUTO_FORM_TSX,
            executable: false,
        },
```

**Step 2: Verify scaffold compiles**

Run: `cd /Users/weiloonso/Projects/personal/Rust/Rustforge && cargo check -p scaffold`
Expected: compiles successfully with no errors.

**Step 3: Commit**

```bash
git add scaffold/src/main.rs
git commit -m "feat: register useAutoForm.tsx in scaffold file templates"
```

---

### Task 6: End-to-end verification — re-scaffold and verify

**Step 1: Re-run scaffold to fresh output**

```bash
cd /Users/weiloonso/Projects/personal/Rust/Rustforge
cargo run -p scaffold -- --output /tmp/rustforge-autoform-test --force
```
Expected: exits 0, all files written.

**Step 2: Verify generated files exist and match**

Check these files exist in `/tmp/rustforge-autoform-test/`:
- `frontend/src/shared/useAutoForm.tsx` — should contain the full hook
- `frontend/src/shared/components/index.ts` — should contain `useAutoForm` re-exports
- `frontend/src/admin/app.css` — should contain `.rf-form-grid`
- `frontend/src/user/app.css` — should contain `.rf-form-grid`

**Step 3: Verify TypeScript in generated project**

```bash
cd /tmp/rustforge-autoform-test/frontend
npm install
npx tsc --noEmit
```
Expected: no type errors.

**Step 4: Clean up test scaffold**

```bash
rm -rf /tmp/rustforge-autoform-test
```

**Step 5: Final commit if any fixes were needed**

If any fixes were required, commit them. Otherwise, this task is just verification.
