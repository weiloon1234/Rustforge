import { useEffect, useCallback, useState } from "react";
import { ChevronLeft, ChevronRight, Download, FileText, Music, X } from "lucide-react";
import { useTranslation } from "react-i18next";

export interface MediaLightboxItem {
  url: string;
  type: "image" | "video" | "audio" | "file";
  caption?: string;
}

export interface MediaLightboxProps {
  items: MediaLightboxItem[];
  index: number;
  onClose: () => void;
  onIndexChange?: (index: number) => void;
}

export function MediaLightbox({
  items,
  index,
  onClose,
  onIndexChange,
}: MediaLightboxProps) {
  const { t } = useTranslation();
  const [currentIndex, setCurrentIndex] = useState(index);

  useEffect(() => {
    setCurrentIndex(index);
  }, [index]);

  const goTo = useCallback(
    (next: number) => {
      setCurrentIndex(next);
      onIndexChange?.(next);
    },
    [onIndexChange],
  );

  const goPrevious = useCallback(() => {
    goTo((currentIndex - 1 + items.length) % items.length);
  }, [currentIndex, items.length, goTo]);

  const goNext = useCallback(() => {
    goTo((currentIndex + 1) % items.length);
  }, [currentIndex, items.length, goTo]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
      if (e.key === "ArrowLeft" && items.length > 1) goPrevious();
      if (e.key === "ArrowRight" && items.length > 1) goNext();
    },
    [onClose, goPrevious, goNext, items.length],
  );

  useEffect(() => {
    if (items.length === 0) return;
    document.addEventListener("keydown", handleKeyDown);
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.body.style.overflow = "";
    };
  }, [items.length, handleKeyDown]);

  if (items.length === 0) return null;

  const current = items[currentIndex];
  if (!current) return null;

  return (
    <div
      className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/85"
      onClick={onClose}
    >
      {/* Close button */}
      <button
        type="button"
        className="absolute right-4 top-4 z-10 rounded-full bg-black/50 p-2 text-white transition-colors hover:bg-black/70"
        onClick={onClose}
      >
        <X size={20} />
      </button>

      {/* Previous button */}
      {items.length > 1 && (
        <button
          type="button"
          className="absolute left-4 z-10 rounded-full bg-black/50 p-2 text-white transition-colors hover:bg-black/70"
          onClick={(e) => {
            e.stopPropagation();
            goPrevious();
          }}
        >
          <ChevronLeft size={24} />
        </button>
      )}

      {/* Media content */}
      <div
        className="flex max-h-[90vh] max-w-[90vw] flex-col items-center"
        onClick={(e) => e.stopPropagation()}
      >
        {current.type === "video" ? (
          <video
            key={current.url}
            src={current.url}
            controls
            autoPlay
            className="max-h-[75vh] max-w-[90vw] rounded-lg"
          />
        ) : current.type === "audio" ? (
          <div className="flex flex-col items-center gap-6 rounded-2xl bg-white/10 px-12 py-10 backdrop-blur-sm">
            <Music size={64} className="text-white/60" />
            <p className="max-w-[40vw] truncate text-lg font-medium text-white">
              {current.caption || "Audio"}
            </p>
            <audio
              key={current.url}
              src={current.url}
              controls
              autoPlay
              className="w-80"
            />
          </div>
        ) : current.type === "file" ? (
          <div className="flex flex-col items-center gap-6 rounded-2xl bg-white/10 px-12 py-10 backdrop-blur-sm">
            <FileText size={64} className="text-white/60" />
            <p className="max-w-[40vw] truncate text-lg font-medium text-white">
              {current.caption || "File"}
            </p>
            <a
              href={current.url}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 rounded-lg bg-white px-5 py-2.5 text-sm font-medium text-[#344054] transition hover:bg-white/90"
            >
              <Download size={16} />
              {t("Download")}
            </a>
          </div>
        ) : (
          <img
            src={current.url}
            alt={current.caption ?? ""}
            className="max-h-[75vh] max-w-[90vw] rounded-lg object-contain"
          />
        )}

        {/* Caption bar — always visible */}
        <div className="mt-3 flex items-center gap-3 text-sm text-white/80">
          {current.caption && <span>{current.caption}</span>}
          {items.length > 1 && (
            <span className="text-white/50">
              {currentIndex + 1} / {items.length}
            </span>
          )}
        </div>
      </div>

      {/* Next button */}
      {items.length > 1 && (
        <button
          type="button"
          className="absolute right-4 z-10 rounded-full bg-black/50 p-2 text-white transition-colors hover:bg-black/70"
          onClick={(e) => {
            e.stopPropagation();
            goNext();
          }}
        >
          <ChevronRight size={24} />
        </button>
      )}
    </div>
  );
}

/** Helper: convert content_type to MediaLightboxItem type */
export function toLightboxType(contentType: string): "image" | "video" | "audio" | "file" {
  if (contentType.startsWith("video/")) return "video";
  if (contentType.startsWith("audio/")) return "audio";
  if (contentType.startsWith("image/")) return "image";
  return "file";
}
