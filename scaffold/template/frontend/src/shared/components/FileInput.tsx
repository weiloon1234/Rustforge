import {
  forwardRef,
  useId,
  useMemo,
  useEffect,
  useState,
  type ChangeEvent,
  type InputHTMLAttributes,
} from "react";
import { FileText } from "lucide-react";
import { useTranslation } from "react-i18next";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export interface FilePreviewItem {
  name: string;
  url?: string;
  mimeType?: string;
  size?: number;
}

export interface FileInputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, "type" | "value" | "defaultValue"> {
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  containerClassName?: string;
  files?: File[];
  defaultFiles?: FilePreviewItem[];
  accepts?: string;
  maxFiles?: number;
}

export const FileInput = forwardRef<HTMLInputElement, FileInputProps>(
  (
    {
      label,
      error,
      errors,
      notes,
      required,
      className,
      containerClassName,
      id: externalId,
      files = [],
      defaultFiles = [],
      accept,
      multiple,
      accepts,
      maxFiles,
      onChange,
      ...rest
    },
    ref,
  ) => {
    const { t } = useTranslation();
    const autoId = useId();
    const id = externalId ?? autoId;
    const [maxFilesWarning, setMaxFilesWarning] = useState<string | null>(null);
    const resolvedAccept = accept ?? accepts;
    const previewItems = useMemo(() => {
      if (files.length > 0) {
        return files.map((file) => ({
          name: file.name,
          mimeType: file.type || undefined,
          size: file.size,
          url: file.type.startsWith("image/") ? URL.createObjectURL(file) : undefined,
          source: "selected" as const,
        }));
      }
      return defaultFiles.map((item) => ({ ...item, source: "default" as const }));
    }, [files, defaultFiles]);

    useEffect(() => {
      return () => {
        for (const item of previewItems) {
          if (item.source === "selected" && item.url) {
            URL.revokeObjectURL(item.url);
          }
        }
      };
    }, [previewItems]);

    const hasPreview = previewItems.length > 0;
    const preview = previewItems[0];
    const isImagePreview =
      !!preview?.url &&
      (preview.mimeType?.startsWith("image/") || /\.(png|jpe?g|gif|webp|svg|bmp|heic|heif)$/i.test(preview.url));

    const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
      const selectedCount = event.target.files?.length ?? 0;
      if (multiple && maxFiles && selectedCount > maxFiles) {
        setMaxFilesWarning(t("Maximum :count files allowed", { count: maxFiles }));
      } else {
        setMaxFilesWarning(null);
      }
      onChange?.(event);
    };

    return (
      <div className={`rf-field ${containerClassName ?? ""}`}>
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={id}
          type="file"
          required={required}
          accept={resolvedAccept}
          multiple={multiple}
          onChange={handleChange}
          className={`rf-input ${hasFieldError(error, errors) ? "rf-input-error" : ""} ${className ?? ""}`}
          {...rest}
        />
        {!hasFieldError(error, errors) && (
          <>
            {!hasPreview && <p className="rf-note">{t("No file selected")}</p>}
            {hasPreview && previewItems.length > 1 && (
              <p className="rf-note">{t(":count files selected", { count: previewItems.length })}</p>
            )}
            {hasPreview && previewItems.length === 1 && preview && (
              <div className="mt-2 flex items-center gap-3 rounded-lg border border-border bg-surface px-3 py-2">
                {isImagePreview ? (
                  <img src={preview.url} alt={preview.name} className="h-12 w-12 rounded object-cover" />
                ) : (
                  <span className="inline-flex h-12 w-12 items-center justify-center rounded bg-surface-hover text-muted">
                    <FileText size={18} />
                  </span>
                )}
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium text-foreground">{preview.name}</p>
                  {typeof preview.size === "number" && (
                    <p className="text-xs text-muted">{preview.size.toLocaleString()} bytes</p>
                  )}
                </div>
              </div>
            )}
          </>
        )}
        <FieldErrors error={error} errors={errors} />
        {maxFilesWarning && !hasFieldError(error, errors) && (
          <p className="text-xs text-amber-500">{maxFilesWarning}</p>
        )}
        {notes && !hasFieldError(error, errors) && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

FileInput.displayName = "FileInput";
