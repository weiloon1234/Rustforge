import { useEffect, type ReactNode } from "react";
import type { ModalEntry, ModalSize } from "@shared/useModalStore";

const sizeClasses: Record<ModalSize, string> = {
  sm: "max-w-sm",
  md: "max-w-md",
  lg: "max-w-lg",
  xl: "max-w-xl",
  full: "max-w-[calc(100vw-2rem)]",
};

interface ModalProps {
  entry: ModalEntry;
  index: number;
  onClose: (id: string) => void;
}

export function Modal({ entry, index, onClose }: ModalProps) {
  useEffect(() => {
    if (!entry.closeOnEsc) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        onClose(entry.id);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [entry.id, entry.closeOnEsc, onClose]);

  return (
    <div
      className="rf-modal-backdrop"
      style={{ zIndex: 40 + index * 10 }}
      onClick={(e) => {
        if (entry.closeOnBackdrop && e.target === e.currentTarget) {
          onClose(entry.id);
        }
      }}
    >
      <div className={`rf-modal-panel ${sizeClasses[entry.size]}`} role="dialog" aria-modal="true">
        <div className="rf-modal-header">
          <h2 className="rf-modal-title">{entry.title}</h2>
          <button className="rf-modal-close" onClick={() => onClose(entry.id)} aria-label="Close">
            <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
            </svg>
          </button>
        </div>
        <div className="rf-modal-body">{entry.content}</div>
        {entry.footer && <div className="rf-modal-footer">{entry.footer}</div>}
      </div>
    </div>
  );
}
