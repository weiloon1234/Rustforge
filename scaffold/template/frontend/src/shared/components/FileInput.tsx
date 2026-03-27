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
import { Download, Eye, FileText, Music, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";
import { Button } from "@shared/components/Button";
import { MediaLightbox, toLightboxType } from "@shared/components/MediaLightbox";
import type { MediaLightboxItem } from "@shared/components/MediaLightbox";
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
  /** Hide the file chooser input — preview-only mode */
  hideInput?: boolean;
  /** Called when user clicks remove on an item (index in combined preview list) */
  onRemove?: (index: number, item: FilePreviewDisplayItem) => void;
}

export interface FilePreviewDisplayItem extends FilePreviewItem {
  source: "selected" | "default";
}

const IMAGE_FILE_NAME_PATTERN = /\.(avif|bmp|gif|heic|heif|ico|jpe?g|png|svg|webp)$/i;
const VIDEO_FILE_NAME_PATTERN = /\.(mp4|webm|ogg|mov|avi|mkv)$/i;
const AUDIO_FILE_NAME_PATTERN = /\.(mp3|wav|ogg|flac|aac|m4a)$/i;

function isImageFile(item: FilePreviewItem): boolean {
  if (item.mimeType?.startsWith("image/")) return true;
  if (IMAGE_FILE_NAME_PATTERN.test(item.name)) return true;
  if (item.url && IMAGE_FILE_NAME_PATTERN.test(item.url)) return true;
  return false;
}

function isVideoFile(item: FilePreviewItem): boolean {
  if (item.mimeType?.startsWith("video/")) return true;
  if (VIDEO_FILE_NAME_PATTERN.test(item.name)) return true;
  if (item.url && VIDEO_FILE_NAME_PATTERN.test(item.url)) return true;
  return false;
}

function isAudioFile(item: FilePreviewItem): boolean {
  if (item.mimeType?.startsWith("audio/")) return true;
  if (AUDIO_FILE_NAME_PATTERN.test(item.name)) return true;
  if (item.url && AUDIO_FILE_NAME_PATTERN.test(item.url)) return true;
  return false;
}

function resolveFileType(item: FilePreviewItem): "image" | "video" | "audio" | "file" {
  if (item.mimeType) return toLightboxType(item.mimeType);
  if (isImageFile(item)) return "image";
  if (isVideoFile(item)) return "video";
  if (isAudioFile(item)) return "audio";
  return "file";
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
      hideInput,
      onRemove,
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
    const [lightboxIndex, setLightboxIndex] = useState(-1);
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

    const lightboxItems: MediaLightboxItem[] = useMemo(
      () =>
        previewItems
          .filter((item) => !!item.url)
          .map((item) => ({
            url: item.url!,
            type: resolveFileType(item),
            caption: item.name,
          })),
      [previewItems],
    );

    const handlePreview = (_item: FilePreviewDisplayItem, index: number) => {
      setLightboxIndex(index);
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
        {!hideInput && (
          <>
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
          </>
        )}
        {!hasFieldErrors && (
          <>
            {hasPreview && (
              <div className={`${hideInput ? "" : "mt-2 "}space-y-2`}>
                {previewItems.map((item, index) => {
                  const fileType = resolveFileType(item);
                  const hasUrl = !!item.url;
                  const key = `${item.source}-${item.name}-${index}`;

                  return (
                    <div key={key} className="flex items-center gap-3 rounded-lg border border-border bg-surface px-3 py-2">
                      <button
                        type="button"
                        className="shrink-0 cursor-pointer overflow-hidden rounded border border-border transition hover:ring-2 hover:ring-primary/40"
                        onClick={() => hasUrl && handlePreview(item, index)}
                        disabled={!hasUrl}
                      >
                        {fileType === "image" && hasUrl ? (
                          <img src={item.url} alt={item.name} className="h-12 w-12 object-cover" />
                        ) : fileType === "video" ? (
                          <span className="flex h-12 w-12 items-center justify-center bg-surface-hover text-xs text-muted">▶ video</span>
                        ) : fileType === "audio" ? (
                          <span className="flex h-12 w-12 items-center justify-center bg-surface-hover text-muted">
                            <Music size={18} />
                          </span>
                        ) : (
                          <span className="flex h-12 w-12 items-center justify-center bg-surface-hover text-muted">
                            <FileText size={18} />
                          </span>
                        )}
                      </button>
                      <div className="min-w-0 flex-1">
                        <p className="truncate text-sm font-medium">{item.name}</p>
                        {typeof item.size === "number" && (
                          <p className="text-xs text-muted">{(item.size / 1024).toFixed(0)} KB</p>
                        )}
                      </div>
                      <div className="flex items-center gap-1">
                        {hasUrl && (
                          <Button
                            type="button"
                            variant="plain"
                            size="xs"
                            className="px-2 text-xs"
                            onClick={() => handlePreview(item, index)}
                          >
                            <Eye size={14} />
                            {t("Preview")}
                          </Button>
                        )}
                        {hasUrl && (
                          <Button
                            type="button"
                            variant="plain"
                            size="xs"
                            className="px-2 text-xs"
                            onClick={() => handleDownload(item)}
                          >
                            <Download size={14} />
                            {t("Download")}
                          </Button>
                        )}
                        {onRemove && (
                          <Button
                            type="button"
                            variant="plain"
                            size="xs"
                            className="px-2 text-xs text-destructive hover:text-destructive"
                            onClick={() => onRemove(index, item)}
                          >
                            <X size={14} />
                            {t("Remove")}
                          </Button>
                        )}
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
        {lightboxIndex >= 0 && lightboxItems.length > 0 && (
          <MediaLightbox
            items={lightboxItems}
            index={lightboxIndex}
            onClose={() => setLightboxIndex(-1)}
            onIndexChange={setLightboxIndex}
          />
        )}
      </div>
    );
  },
);

FileInput.displayName = "FileInput";
