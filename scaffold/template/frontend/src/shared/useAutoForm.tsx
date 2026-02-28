import { useState, useMemo, useCallback, type ReactElement } from "react";
import type { AxiosInstance, AxiosError } from "axios";
import { TextInput } from "@shared/components/TextInput";
import { TextArea } from "@shared/components/TextArea";
import { Select, type SelectOption } from "@shared/components/Select";
import { Checkbox } from "@shared/components/Checkbox";
import { Radio, type RadioOption } from "@shared/components/Radio";

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
  /** Static key-value pairs merged into every submission (not rendered as form fields). */
  extraPayload?: Record<string, unknown>;
  onSuccess?: (data: unknown) => void;
  onError?: (error: unknown) => void;
}

interface AutoFormErrors {
  general: string | null;
  fields: Record<string, string[]>;
}

interface AutoFormResult {
  submit: () => Promise<void>;
  busy: boolean;
  form: ReactElement;
  errors: AutoFormErrors;
  values: Record<string, string>;
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
  const { url, method = "post", fields, defaults, extraPayload, onSuccess, onError } = config;

  const [values, setValuesState] = useState<Record<string, string>>(() => buildDefaults(fields, defaults));
  const [fieldErrors, setFieldErrors] = useState<Record<string, string[]>>({});
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
    const payload: Record<string, unknown> = { ...extraPayload };
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
          const mapped: Record<string, string[]> = {};
          for (const [key, msgs] of Object.entries(body.errors)) {
            if (msgs.length > 0) {
              // Use base field name (strip nested suffixes like ".value")
              const fieldKey = key.split(".")[0];
              mapped[fieldKey] = [...(mapped[fieldKey] ?? []), ...msgs];
            }
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
  }, [api, method, url, fields, values, extraPayload, onSuccess, onError]);

  const form = useMemo((): ReactElement => {
    return (
      <div className="rf-form-grid">
        {fields.map((field) => {
          const span = field.span ?? 2;
          const style = { gridColumn: `span ${span}` };
          const errors = fieldErrors[field.name];

          switch (field.type) {
            case "textarea":
              return (
                <div key={field.name} style={style}>
                  <TextArea
                    label={field.label}
                    value={values[field.name] ?? ""}
                    onChange={(e) => setValue(field.name, e.target.value)}
                    errors={errors}
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
                    errors={errors}
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
                    errors={errors}
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
                    errors={errors}
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
                    errors={errors}
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

  return { submit, busy, form, errors: { general: generalError, fields: fieldErrors }, values, reset, setValues };
}
