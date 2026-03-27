import { useEffect, useCallback } from "react";
import { ChevronLeft, ChevronRight, X } from "lucide-react";

export interface ImageLightboxProps {
  images: string[];
  index: number;
  onClose: () => void;
  onPrevious?: () => void;
  onNext?: () => void;
}

export default function ImageLightbox({
  images,
  index,
  onClose,
  onPrevious,
  onNext,
}: ImageLightboxProps) {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
      if (e.key === "ArrowLeft") onPrevious?.();
      if (e.key === "ArrowRight") onNext?.();
    },
    [onClose, onPrevious, onNext],
  );

  useEffect(() => {
    if (images.length === 0) return;
    document.addEventListener("keydown", handleKeyDown);
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.body.style.overflow = "";
    };
  }, [images.length, handleKeyDown]);

  if (images.length === 0) return null;

  const src = images[index];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/80"
      onClick={onClose}
    >
      <button
        type="button"
        className="absolute right-4 top-4 rounded-full bg-black/50 p-2 text-white transition-colors hover:bg-black/70"
        onClick={onClose}
      >
        <X size={20} />
      </button>

      {onPrevious && images.length > 1 && (
        <button
          type="button"
          className="absolute left-4 rounded-full bg-black/50 p-2 text-white transition-colors hover:bg-black/70"
          onClick={(e) => {
            e.stopPropagation();
            onPrevious();
          }}
        >
          <ChevronLeft size={24} />
        </button>
      )}

      <img
        src={src}
        alt=""
        className="max-h-[90vh] max-w-[90vw] rounded-lg object-contain"
        onClick={(e) => e.stopPropagation()}
      />

      {onNext && images.length > 1 && (
        <button
          type="button"
          className="absolute right-4 rounded-full bg-black/50 p-2 text-white transition-colors hover:bg-black/70"
          onClick={(e) => {
            e.stopPropagation();
            onNext();
          }}
        >
          <ChevronRight size={24} />
        </button>
      )}
    </div>
  );
}
