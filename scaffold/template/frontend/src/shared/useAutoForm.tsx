import { useState, useMemo, useCallback, type ReactElement } from "react";
import type { AxiosInstance, AxiosError } from "axios";
import { TextInput } from "@shared/components/TextInput";
import { TextArea } from "@shared/components/TextArea";
import { Select, type SelectOption } from "@shared/components/Select";
import { Checkbox } from "@shared/components/Checkbox";
import { Radio, type RadioOption } from "@shared/components/Radio";
import { CheckboxGroup, type CheckboxGroupOption } from "@shared/components/CheckboxGroup";
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
import { ContactInput } from "@shared/components/ContactInput";
import { useLocaleStore } from "@shared/stores/locale";
import { useTranslation } from "react-i18next";

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

// Shared props across all field types
type FieldBase = { span?: 1 | 2; required?: boolean; notes?: string; disabled?: boolean; localized?: boolean };

// Type-specific variants (without name — name is added via FieldIdentity)
type FieldVariant =
  | (FieldBase & { type: InputFieldType; label: string; placeholder?: string })
  | (FieldBase & { type: RichEditorFieldType; label: string; placeholder?: string; editorPreset?: TiptapPreset; imageFolder?: string })
  | (FieldBase & { type: "textarea"; label: string; placeholder?: string; rows?: number })
  | (FieldBase & { type: "select"; label: string; options: SelectOption[]; placeholder?: string })
  | (FieldBase & { type: "checkbox"; label: string })
  | (FieldBase & { type: "checkboxGroup"; label: string; options: CheckboxGroupOption[]; columns?: number })
  | (FieldBase & { type: "radio"; label: string; options: RadioOption[] })
  | (FieldBase & { type: "file"; label: string; accept?: string; accepts?: string; multiple?: boolean; maxFiles?: number })
  | (FieldBase & { type: "files"; label: string; accept?: string; accepts?: string; maxFiles?: number });

// Name identity: payload fields must match T keys, virtual fields can be any string
type FieldIdentity<T> =
  | { name: keyof T & string; virtual?: false }
  | { name: string; virtual: true };

// Contact is special: name is an identifier, actual payload keys are countryName/phoneName
type ContactField = FieldBase & {
  name: string;
  type: "contact";
  label?: string;
  virtual?: boolean;
  countryName?: string;
  phoneName?: string;
};

// Children inside a type: "localized" group — regular fields without span/localized
type LocalizedChildField = Omit<FieldVariant, 'span' | 'localized'> & { name: string };

// Compound field: groups children into locale sections (EN block / ZH block)
type LocalizedGroupField = {
  type: "localized";
  children: LocalizedChildField[];
  span?: 1 | 2;
};

// FieldDef<T>: intersection distributes over both unions → full type-safe field definitions
type FieldDef<T = Record<string, unknown>> = (FieldVariant & FieldIdentity<T>) | ContactField | LocalizedGroupField;

interface AutoFormConfig<T = Record<string, unknown>> {
  url: string;
  method?: "post" | "put" | "patch";
  bodyType?: AutoFormBodyType;
  fields: FieldDef<NoInfer<T>>[] | ((values: Record<string, string>) => FieldDef<NoInfer<T>>[]);
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
  LocalizedGroupField,
  LocalizedChildField,
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

function isLocalizedGroup(field: FieldDef<any>): field is LocalizedGroupField {
  return field.type === "localized";
}

function isLocalizable(field: FieldDef<any>): field is FieldDef<any> & { localized: true } {
  return "localized" in field && field.localized === true;
}

function collectLocalizedFieldNames(fields: FieldDef<any>[]): Set<string> {
  const names = new Set<string>();
  for (const field of fields) {
    if (isLocalizedGroup(field)) {
      for (const child of field.children) names.add(child.name);
    } else if (isLocalizable(field)) {
      names.add(field.name);
    }
  }
  return names;
}

function buildDefaults(fields: FieldDef<any>[], locales: string[], defaults?: Record<string, AutoFormDefaultValue>): Record<string, string> {
  const values: Record<string, string> = {};
  for (const field of fields) {
    // Pattern 2: type: "localized" with children
    if (isLocalizedGroup(field)) {
      for (const child of field.children) {
        const raw = defaults?.[child.name];
        const localeDefaults = typeof raw === "object" && raw !== null && !Array.isArray(raw)
          ? raw as Record<string, unknown> : undefined;
        if (child.type === "file" || child.type === "files") {
          for (const locale of locales) {
            const ld = localeDefaults?.[locale];
            if (typeof ld === "string") values[`${child.name}.${locale}`] = ld;
            else if (ld && typeof ld === "object" && "name" in (ld as any)) values[`${child.name}.${locale}`] = (ld as any).name ?? "";
            else values[`${child.name}.${locale}`] = "";
          }
        } else {
          for (const locale of locales) {
            values[`${child.name}.${locale}`] = toTextDefault(localeDefaults?.[locale] as AutoFormDefaultValue);
          }
        }
      }
      continue;
    }
    // Pattern 1: localized: true on individual field (handles file + text types)
    if (isLocalizable(field)) {
      const raw = defaults?.[field.name];
      const localeDefaults = typeof raw === "object" && raw !== null && !Array.isArray(raw)
        ? raw as Record<string, unknown> : undefined;
      if (field.type === "file" || field.type === "files") {
        for (const locale of locales) {
          const ld = localeDefaults?.[locale];
          if (typeof ld === "string") values[`${field.name}.${locale}`] = ld;
          else if (ld && typeof ld === "object" && "name" in (ld as any)) values[`${field.name}.${locale}`] = (ld as any).name ?? "";
          else values[`${field.name}.${locale}`] = "";
        }
      } else {
        for (const locale of locales) {
          values[`${field.name}.${locale}`] = toTextDefault(localeDefaults?.[locale] as AutoFormDefaultValue);
        }
      }
      continue;
    }
    if (field.type === "file" || field.type === "files") {
      values[field.name] = "";
      continue;
    }
    if (field.type === "checkboxGroup") {
      const raw = defaults?.[field.name];
      values[field.name] = Array.isArray(raw) ? JSON.stringify(raw) : (typeof raw === "string" ? raw : "[]");
      continue;
    }
    if (field.type === "contact") {
      const cKey = field.countryName ?? "country_iso2";
      const pKey = field.phoneName ?? "contact_number";
      values[cKey] = toTextDefault(defaults?.[cKey]);
      values[pKey] = toTextDefault(defaults?.[pKey]);
      continue;
    }
    values[field.name] = toTextDefault(defaults?.[field.name]);
  }
  return values;
}

function buildFileDefaultPreviews(
  fields: FieldDef<any>[],
  defaults?: Record<string, AutoFormDefaultValue>,
): Record<string, FilePreviewItem[]> {
  const result: Record<string, FilePreviewItem[]> = {};
  if (!defaults) return result;

  for (const field of fields) {
    // Pattern 2: type: "localized" with file children
    if (isLocalizedGroup(field)) {
      for (const child of field.children) {
        if (child.type !== "file" && child.type !== "files") continue;
        const raw = defaults[child.name];
        if (!raw || typeof raw !== "object" || Array.isArray(raw)) continue;
        for (const [locale, localeValue] of Object.entries(raw as Record<string, unknown>)) {
          const preview = normalizeFilePreview(localeValue as AutoFormDefaultValue);
          if (preview) result[`${child.name}.${locale}`] = [preview];
        }
      }
      continue;
    }

    // Pattern 1: localized: true on file field
    if ((field.type === "file" || field.type === "files") && isLocalizable(field)) {
      const raw = defaults[field.name];
      if (!raw || typeof raw !== "object" || Array.isArray(raw)) continue;
      for (const [locale, localeValue] of Object.entries(raw as Record<string, unknown>)) {
        const preview = normalizeFilePreview(localeValue as AutoFormDefaultValue);
        if (preview) result[`${field.name}.${locale}`] = [preview];
      }
      continue;
    }

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

  // Flatten plain objects to dot-key notation (e.g. title.en=Hi)
  if (typeof value === "object" && value !== null && !Array.isArray(value) && !(value instanceof Date)) {
    for (const [subKey, subValue] of Object.entries(value as Record<string, unknown>)) {
      appendFormDataValue(formData, `${key}.${subKey}`, subValue);
    }
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

function shouldSerializeEmptyAsNull(field: FieldDef<any>): boolean {
  if (field.required) return false;
  return field.type !== "checkbox" && field.type !== "checkboxGroup" && field.type !== "file" && field.type !== "files";
}

export function useAutoForm<T = Record<string, unknown>>(api: AxiosInstance, config: AutoFormConfig<T>): AutoFormResult {
  const {
    url,
    method = "post",
    bodyType = "auto",
    fields: fieldsProp,
    defaults,
    tiptapImageUpload,
    extraPayload,
    onSuccess,
    onError,
  } = config;

  const { t } = useTranslation();
  const availableLocales = useLocaleStore((s) => s.availableLocales);
  const resolveFields = typeof fieldsProp === "function" ? fieldsProp : () => fieldsProp;
  const [values, setValuesState] = useState<Record<string, string>>(() => {
    const initialFields = resolveFields({});
    return buildDefaults(initialFields, availableLocales, defaults);
  });

  // Resolve fields reactively from current values
  const fields = resolveFields(values);

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
    const resetFields = resolveFields({});
    setValuesState(buildDefaults(resetFields, availableLocales, defaults));
    setFileValues({});
    setFileInputVersion((prev) => prev + 1);
    setFieldErrors({});
    setGeneralError(null);
  }, [resolveFields, availableLocales, defaults]);

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
      if ("virtual" in field && field.virtual) continue;

      // Pattern 2: type: "localized" with children
      if (isLocalizedGroup(field)) {
        for (const child of field.children) {
          if (child.type === "file" || child.type === "files") {
            const localeValues: Record<string, unknown> = {};
            for (const locale of availableLocales) {
              const fileKey = `${child.name}.${locale}`;
              const selected = fileValues[fileKey] ?? [];
              if (selected.length > 0) {
                localeValues[locale] = selected[0];
              } else {
                const existing = (values[fileKey] ?? "").trim();
                localeValues[locale] = existing || null;
              }
            }
            payload[child.name] = localeValues;
          } else {
            const localeValues: Record<string, string | null> = {};
            for (const locale of availableLocales) {
              const v = (values[`${child.name}.${locale}`] ?? "").trim();
              localeValues[locale] = v || null;
            }
            payload[child.name] = localeValues;
          }
        }
        continue;
      }

      // Pattern 1: localized: true on individual field
      if (isLocalizable(field)) {
        if (field.type === "file" || field.type === "files") {
          const localeValues: Record<string, unknown> = {};
          for (const locale of availableLocales) {
            const fileKey = `${field.name}.${locale}`;
            const selected = fileValues[fileKey] ?? [];
            if (selected.length > 0) {
              localeValues[locale] = selected[0];
            } else {
              const existing = (values[fileKey] ?? "").trim();
              localeValues[locale] = existing || null;
            }
          }
          payload[field.name] = localeValues;
        } else {
          const localeValues: Record<string, string | null> = {};
          for (const locale of availableLocales) {
            const v = (values[`${field.name}.${locale}`] ?? "").trim();
            localeValues[locale] = v || null;
          }
          payload[field.name] = localeValues;
        }
        continue;
      }

      if (field.type === "file" || field.type === "files") {
        const selected = fileValues[field.name] ?? [];
        if (selected.length === 0) continue;
        payload[field.name] = field.type === "file" && !field.multiple ? selected[0] : selected;
        continue;
      }

      if (field.type === "contact") {
        const cKey = field.countryName ?? "country_iso2";
        const pKey = field.phoneName ?? "contact_number";
        const cVal = (values[cKey] ?? "").trim();
        const pVal = (values[pKey] ?? "").trim();
        payload[cKey] = cVal === "" ? null : cVal;
        payload[pKey] = pVal === "" ? null : pVal;
        continue;
      }

      const value = values[field.name] ?? "";
      if (field.type === "checkboxGroup") {
        try { payload[field.name] = JSON.parse(value || "[]"); } catch { payload[field.name] = []; }
        continue;
      }
      if (field.type === "checkbox") {
        payload[field.name] = value === "1";
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
          const localizedNames = collectLocalizedFieldNames(fields);
          const mapped: Record<string, string[]> = {};
          for (const [key, msgs] of Object.entries(body.errors)) {
            if (msgs.length > 0) {
              // Preserve locale suffix for localized fields (title.en → title.en)
              // Collapse nested suffixes for non-localized fields (address.line1 → address)
              const baseName = key.split(".")[0];
              const fieldKey = localizedNames.has(baseName) ? key : baseName;
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
  }, [api, method, url, bodyType, fields, values, fileValues, availableLocales, extraPayload, onSuccess, onError, busy]);

  const renderLocalizedField = useCallback((field: FieldDef<T>, locale: string): ReactElement => {
    const valueKey = `${field.name}.${locale}`;
    const errors = fieldErrors[valueKey];
    const localeName = t(`Locale ${locale.toUpperCase()}`);
    const label = `${field.label} (${localeName})`;

    switch (field.type) {
      case "textarea":
        return (
          <TextArea
            label={label}
            value={values[valueKey] ?? ""}
            onChange={(e) => setValue(valueKey, e.target.value)}
            errors={errors}
            notes={field.notes}
            placeholder={(field as { placeholder?: string }).placeholder}
            required={field.required}
            disabled={field.disabled}
            rows={field.rows}
          />
        );
      case "tapbit":
      case "tiptap":
        return (
          <TiptapInput
            label={label}
            value={values[valueKey] ?? ""}
            onChange={(e) => setValue(valueKey, e.target.value)}
            errors={errors}
            notes={field.notes}
            preset={(field as { editorPreset?: TiptapPreset }).editorPreset}
            placeholder={(field as { placeholder?: string }).placeholder}
            imageFolder={(field as { imageFolder?: string }).imageFolder}
            imageUpload={tiptapImageUpload}
            required={field.required}
            disabled={field.disabled}
          />
        );
      case "file":
      case "files": {
        const fileKey = `${field.name}.${locale}`;
        return (
          <FileInput
            label={label}
            accept={(field as any).accept ?? (field as any).accepts}
            multiple={field.type === "files" || (field as any).multiple}
            maxFiles={(field as any).maxFiles}
            files={fileValues[fileKey] ?? []}
            defaultFiles={defaultFilePreviews[fileKey] ?? []}
            onChange={(e) => setFiles(fileKey, Array.from(e.target.files ?? []), (field as any).maxFiles)}
            errors={errors}
            notes={field.notes}
            required={field.required}
            disabled={field.disabled}
          />
        );
      }
      case "select":
        return (
          <Select
            label={label}
            options={(field as any).options}
            value={values[valueKey] ?? ""}
            onChange={(e) => setValue(valueKey, e.target.value)}
            errors={errors}
            notes={field.notes}
            placeholder={(field as any).placeholder}
            required={field.required}
            disabled={field.disabled}
          />
        );
      default: {
        const inputField = field as FieldDef<T> & { type: InputFieldType };
        return (
          <TextInput
            type={inputField.type}
            label={label}
            value={values[valueKey] ?? ""}
            onChange={(e) => setValue(valueKey, e.target.value)}
            errors={errors}
            notes={field.notes}
            placeholder={(field as { placeholder?: string }).placeholder}
            required={field.required}
            disabled={field.disabled}
          />
        );
      }
    }
  }, [values, fieldErrors, fileValues, defaultFilePreviews, setValue, setFiles, tiptapImageUpload]);

  const form = useMemo((): ReactElement => {
    return (
      <div className="rf-form-grid">
        {fields.map((field) => {
          // Pattern 2: type: "localized" — grouped locale sections
          if (isLocalizedGroup(field)) {
            const groupSpan = field.span ?? 2;
            const groupStyle = { gridColumn: `span ${groupSpan}` };
            return (
              <div key={`localized-${field.children.map(c => c.name).join("-")}`} style={groupStyle} className="space-y-4">
                {availableLocales.map((locale) => (
                  <div key={locale} className="space-y-3 rounded-lg border border-border bg-surface px-4 py-4">
                    <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                      {t(`Locale ${locale.toUpperCase()}`)}
                    </p>
                    {field.children.map((child) => (
                      <div key={child.name}>
                        {renderLocalizedField(child as FieldDef<T>, locale)}
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            );
          }

          const span = field.span ?? 2;
          const style = { gridColumn: `span ${span}` };
          const errors = fieldErrors[field.name];

          // Pattern 1: localized: true — inline locale cards
          if (isLocalizable(field)) {
            return (
              <div key={field.name} style={style} className="space-y-3">
                {availableLocales.map((locale) => (
                  <div key={locale} className="rounded-md border border-border p-3">
                    <div className="mb-2 text-xs font-medium text-muted-foreground tracking-wide">{t(`Locale ${locale.toUpperCase()}`)}</div>
                    {renderLocalizedField(field, locale)}
                  </div>
                ))}
              </div>
            );
          }

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

            case "checkboxGroup": {
              const parsed: string[] = (() => { try { return JSON.parse(values[field.name] || "[]"); } catch { return []; } })();
              return (
                <div key={field.name} style={style}>
                  <CheckboxGroup
                    name={field.name}
                    label={field.label}
                    options={field.options}
                    value={parsed}
                    onChange={(next) => setValue(field.name, JSON.stringify(next))}
                    errors={errors}
                    notes={field.notes}
                    required={field.required}
                    disabled={field.disabled}
                    columns={field.columns}
                  />
                </div>
              );
            }

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

            case "contact": {
              const cKey = field.countryName ?? "country_iso2";
              const pKey = field.phoneName ?? "contact_number";
              return (
                <div key={field.name} style={style}>
                  <ContactInput
                    label={field.label}
                    countryName={cKey}
                    phoneName={pKey}
                    value={{
                      country_iso2: values[cKey] ?? "",
                      phone_number: values[pKey] ?? "",
                    }}
                    onChange={(v) => {
                      setValue(cKey, v.country_iso2);
                      setValue(pKey, v.phone_number);
                    }}
                    countryErrors={fieldErrors[cKey]}
                    phoneErrors={fieldErrors[pKey]}
                    required={field.required}
                    disabled={field.disabled}
                  />
                </div>
              );
            }

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
              const inputField = field as FieldDef<T> & { type: InputFieldType };
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
    availableLocales,
    setValue,
    setFiles,
    tiptapImageUpload,
    renderLocalizedField,
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
