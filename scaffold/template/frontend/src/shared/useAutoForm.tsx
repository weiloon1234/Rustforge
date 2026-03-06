import { useState, useMemo, useCallback, type ReactElement } from "react";
import type { AxiosInstance, AxiosError } from "axios";
import { TextInput } from "@shared/components/TextInput";
import { TextArea } from "@shared/components/TextArea";
import { Select, type SelectOption } from "@shared/components/Select";
import { Checkbox } from "@shared/components/Checkbox";
import { Radio, type RadioOption } from "@shared/components/Radio";
import {
  DatePickerInput,
  DateTimePickerInput,
  TimePickerInput,
} from "@shared/components/TemporalInput";
import {
  TiptapInput,
  type TiptapImageUploadHandler,
  type TiptapPreset,
} from "@shared/components/TiptapInput";
import { FileInput, type FilePreviewItem } from "@shared/components/FileInput";

type InputFieldType =
  | "text"
  | "email"
  | "password"
  | "search"
  | "url"
  | "tel"
  | "number"
  | "money"
  | "atm"
  | "pin"
  | "date"
  | "datetime-local"
  | "time";
type RichEditorFieldType = "tapbit" | "tiptap";
type AutoFormBodyType = "auto" | "json" | "multipart";
type AutoFormDefaultValue = string | number | boolean | null | undefined | File | File[] | FilePreviewItem | FilePreviewItem[];

type FieldDef =
  | { name: string; type: InputFieldType; label: string; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean }
  | {
      name: string;
      type: RichEditorFieldType;
      label: string;
      span?: 1 | 2;
      required?: boolean;
      notes?: string;
      placeholder?: string;
      disabled?: boolean;
      editorPreset?: TiptapPreset;
      imageFolder?: string;
    }
  | { name: string; type: "textarea"; label: string; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean; rows?: number }
  | { name: string; type: "select"; label: string; options: SelectOption[]; span?: 1 | 2; required?: boolean; notes?: string; placeholder?: string; disabled?: boolean }
  | { name: string; type: "checkbox"; label: string; span?: 1 | 2; required?: boolean; notes?: string; disabled?: boolean }
  | { name: string; type: "radio"; label: string; options: RadioOption[]; span?: 1 | 2; required?: boolean; notes?: string; disabled?: boolean }
  | {
      name: string;
      type: "file";
      label: string;
      span?: 1 | 2;
      required?: boolean;
      notes?: string;
      disabled?: boolean;
      accept?: string;
      accepts?: string;
      multiple?: boolean;
      maxFiles?: number;
    }
  | {
      name: string;
      type: "files";
      label: string;
      span?: 1 | 2;
      required?: boolean;
      notes?: string;
      disabled?: boolean;
      accept?: string;
      accepts?: string;
      maxFiles?: number;
    };

interface AutoFormConfig {
  url: string;
  method?: "post" | "put" | "patch";
  bodyType?: AutoFormBodyType;
  fields: FieldDef[];
  defaults?: Record<string, AutoFormDefaultValue>;
  tiptapImageUpload?: TiptapImageUploadHandler;
  /** Static key-value pairs merged into every submission (not rendered as form fields). */
  extraPayload?: Record<string, unknown>;
  onSuccess?: (data: unknown) => void | Promise<void>;
  onError?: (error: unknown) => void | Promise<void>;
}

interface AutoFormErrors {
  general: string | null;
  fields: Record<string, string[]>;
}

interface AutoFormResult {
  submit: (event?: React.FormEvent<HTMLFormElement>) => Promise<void>;
  busy: boolean;
  form: ReactElement;
  errors: AutoFormErrors;
  values: Record<string, string>;
  fileValues: Record<string, File[]>;
  reset: () => void;
  setValues: (values: Record<string, string>) => void;
}

export type {
  FieldDef,
  AutoFormConfig,
  AutoFormErrors,
  AutoFormResult,
  AutoFormBodyType,
  AutoFormDefaultValue,
};

function toTextDefault(value: AutoFormDefaultValue): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  return "";
}

function toFileNameFromPath(value: string): string {
  const clean = value.split("?")[0].split("#")[0];
  const parts = clean.split("/");
  const last = parts[parts.length - 1];
  return last && last.trim().length > 0 ? decodeURIComponent(last) : value;
}

function normalizeFilePreview(value: AutoFormDefaultValue): FilePreviewItem | null {
  if (value === null || value === undefined) return null;
  if (typeof value === "string") {
    const url = value.trim();
    if (!url) return null;
    const hasUrlLikePrefix = /^(https?:\/\/|\/|data:|blob:)/i.test(url);
    return {
      name: toFileNameFromPath(url),
      url: hasUrlLikePrefix ? url : undefined,
    };
  }
  if (typeof File !== "undefined" && value instanceof File) {
    return {
      name: value.name,
      mimeType: value.type || undefined,
      size: value.size,
    };
  }
  if (typeof value === "object" && !Array.isArray(value)) {
    const record = value as unknown as Record<string, unknown>;
    const url = typeof record.url === "string" && record.url.trim().length > 0 ? record.url : undefined;
    const nameFromRecord = typeof record.name === "string" && record.name.trim().length > 0 ? record.name : undefined;
    const mimeType = typeof record.mimeType === "string" && record.mimeType.trim().length > 0 ? record.mimeType : undefined;
    const size = typeof record.size === "number" && Number.isFinite(record.size) ? record.size : undefined;
    const name = nameFromRecord ?? (url ? toFileNameFromPath(url) : "file");
    return {
      name,
      url,
      mimeType,
      size,
    };
  }
  return null;
}

function buildDefaults(fields: FieldDef[], defaults?: Record<string, AutoFormDefaultValue>): Record<string, string> {
  const values: Record<string, string> = {};
  for (const field of fields) {
    if (field.type === "file" || field.type === "files") {
      values[field.name] = "";
      continue;
    }
    values[field.name] = toTextDefault(defaults?.[field.name]);
  }
  return values;
}

function buildFileDefaultPreviews(
  fields: FieldDef[],
  defaults?: Record<string, AutoFormDefaultValue>,
): Record<string, FilePreviewItem[]> {
  const result: Record<string, FilePreviewItem[]> = {};
  if (!defaults) return result;

  for (const field of fields) {
    if (field.type !== "file" && field.type !== "files") continue;
    const raw = defaults[field.name];
    if (raw === undefined || raw === null) continue;

    const items = Array.isArray(raw) ? raw : [raw];
    const previews = items
      .map((item) => normalizeFilePreview(item))
      .filter((item): item is FilePreviewItem => item !== null);

    if (previews.length > 0) {
      result[field.name] = previews;
    }
  }
  return result;
}

function isBinaryValue(value: unknown): value is File | Blob {
  if (typeof File !== "undefined" && value instanceof File) return true;
  if (typeof Blob !== "undefined" && value instanceof Blob) return true;
  return false;
}

function hasBinary(value: unknown): boolean {
  if (isBinaryValue(value)) return true;
  if (Array.isArray(value)) return value.some((item) => hasBinary(item));
  if (value && typeof value === "object") {
    for (const nested of Object.values(value as Record<string, unknown>)) {
      if (hasBinary(nested)) return true;
    }
  }
  return false;
}

function appendFormDataValue(formData: FormData, key: string, value: unknown): void {
  if (value === undefined) return;
  if (value === null) {
    formData.append(key, "");
    return;
  }

  if (isBinaryValue(value)) {
    formData.append(key, value);
    return;
  }

  if (Array.isArray(value)) {
    for (const item of value) {
      // Repeated key style: files=file1, files=file2
      appendFormDataValue(formData, key, item);
    }
    return;
  }

  if (value instanceof Date) {
    formData.append(key, value.toISOString());
    return;
  }

  switch (typeof value) {
    case "string":
      formData.append(key, value);
      return;
    case "number":
    case "boolean":
    case "bigint":
      formData.append(key, String(value));
      return;
    default:
      formData.append(key, JSON.stringify(value));
      return;
  }
}

function shouldSerializeEmptyAsNull(field: FieldDef): boolean {
  if (field.required) return false;
  return field.type !== "checkbox" && field.type !== "file" && field.type !== "files";
}

export function useAutoForm(api: AxiosInstance, config: AutoFormConfig): AutoFormResult {
  const {
    url,
    method = "post",
    bodyType = "auto",
    fields,
    defaults,
    tiptapImageUpload,
    extraPayload,
    onSuccess,
    onError,
  } = config;

  const [values, setValuesState] = useState<Record<string, string>>(() => buildDefaults(fields, defaults));
  const [fileValues, setFileValues] = useState<Record<string, File[]>>({});
  const [fieldErrors, setFieldErrors] = useState<Record<string, string[]>>({});
  const [generalError, setGeneralError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [fileInputVersion, setFileInputVersion] = useState(0);
  const defaultFilePreviews = useMemo(
    () => buildFileDefaultPreviews(fields, defaults),
    [fields, defaults],
  );

  const setValue = useCallback((name: string, value: string) => {
    setValuesState((prev) => ({ ...prev, [name]: value }));
    setFieldErrors((prev) => {
      if (!prev[name]) return prev;
      const next = { ...prev };
      delete next[name];
      return next;
    });
  }, []);

  const setFiles = useCallback((name: string, files: File[], maxFiles?: number) => {
    const bounded = maxFiles && maxFiles > 0 ? files.slice(0, maxFiles) : files;
    setFileValues((prev) => ({ ...prev, [name]: bounded }));
    setFieldErrors((prev) => {
      if (!prev[name]) return prev;
      const next = { ...prev };
      delete next[name];
      return next;
    });
  }, []);

  const reset = useCallback(() => {
    setValuesState(buildDefaults(fields, defaults));
    setFileValues({});
    setFileInputVersion((prev) => prev + 1);
    setFieldErrors({});
    setGeneralError(null);
  }, [fields, defaults]);

  const setValues = useCallback((incoming: Record<string, string>) => {
    setValuesState((prev) => ({ ...prev, ...incoming }));
  }, []);

  const submit = useCallback(async (event?: React.FormEvent<HTMLFormElement>) => {
    event?.preventDefault();
    event?.stopPropagation();
    if (busy) return;

    setBusy(true);
    setFieldErrors({});
    setGeneralError(null);

    // Build payload — checkboxes send "1"/"0" instead of "on"/""
    const payload: Record<string, unknown> = { ...extraPayload };
    for (const field of fields) {
      if (field.type === "file" || field.type === "files") {
        const selected = fileValues[field.name] ?? [];
        if (selected.length === 0) continue;
        payload[field.name] = field.type === "file" && !field.multiple ? selected[0] : selected;
        continue;
      }

      const value = values[field.name] ?? "";
      if (field.type === "checkbox") {
        payload[field.name] = value ? "1" : "0";
        continue;
      }

      if (shouldSerializeEmptyAsNull(field) && value.trim() === "") {
        payload[field.name] = null;
        continue;
      }

      payload[field.name] = value;
    }

    let requestBody: Record<string, unknown> | FormData = payload;
    if (bodyType === "multipart" || (bodyType === "auto" && hasBinary(payload))) {
      const formData = new FormData();
      for (const [key, value] of Object.entries(payload)) {
        appendFormDataValue(formData, key, value);
      }
      requestBody = formData;
    }

    try {
      const response = await api[method](url, requestBody);
      await onSuccess?.(response.data?.data ?? response.data);
    } catch (err) {
      const axiosErr = err as AxiosError<{ message?: string; errors?: Record<string, string[]> }>;
      const body = axiosErr.response?.data;
      if (body) {
        const message = body.message ?? "Something went wrong";
        setGeneralError(message);
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
        setGeneralError(axiosErr.message || "Something went wrong");
      }
      await onError?.(err);
    } finally {
      setBusy(false);
    }
  }, [api, method, url, bodyType, fields, values, fileValues, extraPayload, onSuccess, onError, busy]);

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

            case "date":
              return (
                <div key={field.name} style={style}>
                  <DatePickerInput
                    label={field.label}
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

            case "datetime-local":
              return (
                <div key={field.name} style={style}>
                  <DateTimePickerInput
                    label={field.label}
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

            case "time":
              return (
                <div key={field.name} style={style}>
                  <TimePickerInput
                    label={field.label}
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

            case "tapbit":
            case "tiptap":
              return (
                <div key={field.name} style={style}>
                  <TiptapInput
                    label={field.label}
                    value={values[field.name] ?? ""}
                    onChange={(e) => setValue(field.name, e.target.value)}
                    errors={errors}
                    notes={field.notes}
                    preset={field.editorPreset}
                    placeholder={field.placeholder}
                    imageFolder={field.imageFolder}
                    imageUpload={tiptapImageUpload}
                    required={field.required}
                    disabled={field.disabled}
                  />
                </div>
              );

            case "file":
            case "files":
              return (
                <div key={`${field.name}-${fileInputVersion}`} style={style}>
                  <FileInput
                    label={field.label}
                    accept={field.accept ?? field.accepts}
                    accepts={field.accepts}
                    multiple={field.type === "files" || field.multiple}
                    maxFiles={field.maxFiles}
                    files={fileValues[field.name] ?? []}
                    defaultFiles={defaultFilePreviews[field.name] ?? []}
                    onChange={(e) =>
                      setFiles(field.name, Array.from(e.target.files ?? []), field.maxFiles)
                    }
                    errors={errors}
                    notes={field.notes}
                    required={field.required}
                    disabled={field.disabled}
                  />
                </div>
              );

            default: {
              // All TextInput types: text, email, password, search, url, tel, number, money, atm, pin
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
  }, [
    fields,
    values,
    fileValues,
    fieldErrors,
    defaultFilePreviews,
    fileInputVersion,
    setValue,
    setFiles,
    tiptapImageUpload,
  ]);

  return {
    submit,
    busy,
    form,
    errors: { general: generalError, fields: fieldErrors },
    values,
    fileValues,
    reset,
    setValues,
  };
}
