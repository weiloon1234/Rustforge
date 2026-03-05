import {
  forwardRef,
  useId,
  useMemo,
  useEffect,
  useRef,
  useState,
  type ChangeEvent,
  type InputHTMLAttributes,
} from "react";
import { Download, Eye, FileText } from "lucide-react";
import { useTranslation } from "react-i18next";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";
import { Button } from "@shared/components/Button";
import { attachmentUrl } from "@shared/helpers";

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

interface FilePreviewDisplayItem extends FilePreviewItem {
  source: "selected" | "default";
}

const IMAGE_FILE_NAME_PATTERN = /\.(avif|bmp|gif|heic|heif|ico|jpe?g|png|svg|webp)$/i;

function isImageFile(item: FilePreviewItem): boolean {
  if (item.mimeType?.startsWith("image/")) return true;
  if (IMAGE_FILE_NAME_PATTERN.test(item.name)) return true;
  if (item.url && IMAGE_FILE_NAME_PATTERN.test(item.url)) return true;
  return false;
}

function resolveDefaultPreviewUrl(item: FilePreviewItem): string | undefined {
  if (item.url?.trim()) return item.url.trim();
  if (!item.name.trim()) return undefined;
  return attachmentUrl(item.name);
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
      disabled,
      onChange,
      ...rest
    },
    ref,
  ) => {
    const { t } = useTranslation();
    const autoId = useId();
    const id = externalId ?? autoId;
    const inputRef = useRef<HTMLInputElement | null>(null);
    const [maxFilesWarning, setMaxFilesWarning] = useState<string | null>(null);
    const resolvedAccept = accept ?? accepts;
    const previewItems = useMemo(() => {
      if (files.length > 0) {
        return files.map((file) => ({
          name: file.name,
          mimeType: file.type || undefined,
          size: file.size,
          url: URL.createObjectURL(file),
          source: "selected" as const,
        }));
      }
      return defaultFiles.map((item) => ({
        ...item,
        url: resolveDefaultPreviewUrl(item),
        source: "default" as const,
      }));
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
    const hasFieldErrors = hasFieldError(error, errors);

    const handlePreview = (item: FilePreviewDisplayItem) => {
      if (!item.url || typeof window === "undefined") return;
      window.open(item.url, "_blank", "noopener,noreferrer");
    };

    const handleDownload = (item: FilePreviewDisplayItem) => {
      if (!item.url || typeof document === "undefined") return;
      const anchor = document.createElement("a");
      anchor.href = item.url;
      anchor.download = item.name;
      anchor.rel = "noopener noreferrer";
      anchor.style.display = "none";
      document.body.appendChild(anchor);
      anchor.click();
      document.body.removeChild(anchor);
    };

    const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
      const selectedCount = event.target.files?.length ?? 0;
      if (multiple && maxFiles && selectedCount > maxFiles) {
        setMaxFilesWarning(t("Maximum :count files allowed", { count: maxFiles }));
      } else {
        setMaxFilesWarning(null);
      }
      onChange?.(event);
    };

    const selectLabel = multiple ? t("Choose files") : t("Choose file");
    const selectedSummary = !hasPreview
      ? t("No file selected")
      : previewItems.length === 1
        ? previewItems[0]?.name ?? t("No file selected")
        : t(":count files selected", { count: previewItems.length });

    return (
      <div className={`rf-field ${containerClassName ?? ""}`}>
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <div
          className={`rf-input flex items-center gap-2 ${hasFieldErrors ? "rf-input-error" : ""} ${className ?? ""}`}
        >
          <Button
            variant="secondary"
            size="xs"
            className="rounded-md"
            onClick={() => inputRef.current?.click()}
            disabled={disabled}
          >
            {selectLabel}
          </Button>
          <p className="min-w-0 flex-1 truncate text-sm text-muted">{selectedSummary}</p>
        </div>
        <input
          ref={(node) => {
            inputRef.current = node;
            if (typeof ref === "function") {
              ref(node);
            } else if (ref) {
              ref.current = node;
            }
          }}
          id={id}
          type="file"
          required={required}
          accept={resolvedAccept}
          multiple={multiple}
          disabled={disabled}
          onChange={handleChange}
          className="sr-only"
          {...rest}
        />
        {!hasFieldErrors && (
          <>
            {hasPreview && (
              <div className="mt-2 space-y-2">
                {previewItems.map((item, index) => {
                  const imagePreview = isImageFile(item) && !!item.url;
                  const canDownload = !!item.url;
                  const key = `${item.source}-${item.name}-${index}`;

                  return (
                    <div key={key} className="flex items-center gap-3 rounded-lg border border-border bg-surface px-3 py-2">
                      {imagePreview ? (
                        <a
                          href={item.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="overflow-hidden rounded border border-border"
                          aria-label={t("Preview")}
                        >
                          <img src={item.url} alt={item.name} className="h-12 w-12 object-cover" />
                        </a>
                      ) : (
                        <span className="inline-flex h-12 w-12 shrink-0 items-center justify-center rounded bg-surface-hover text-muted">
                          <FileText size={18} />
                        </span>
                      )}
                      <div className="min-w-0 flex-1">
                        <p className="truncate text-sm font-medium">{item.name}</p>
                        {typeof item.size === "number" && (
                          <p className="text-xs text-muted">{item.size.toLocaleString()} bytes</p>
                        )}
                      </div>
                      <div className="flex items-center gap-1">
                        {imagePreview && (
                          <Button
                            type="button"
                            variant="plain"
                            size="xs"
                            className="px-2 text-xs"
                            onClick={() => handlePreview(item)}
                          >
                            <Eye size={14} />
                            {t("Preview")}
                          </Button>
                        )}
                        <Button
                          type="button"
                          variant="plain"
                          size="xs"
                          className="px-2 text-xs"
                          onClick={() => handleDownload(item)}
                          disabled={!canDownload}
                        >
                          <Download size={14} />
                          {t("Download")}
                        </Button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </>
        )}
        <FieldErrors error={error} errors={errors} />
        {maxFilesWarning && !hasFieldErrors && (
          <p className="text-xs text-amber-500">{maxFilesWarning}</p>
        )}
        {notes && !hasFieldErrors && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

FileInput.displayName = "FileInput";
