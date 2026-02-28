import { create } from "zustand";
import type { ReactNode } from "react";

export type ModalSize = "sm" | "md" | "lg" | "xl" | "full";

export interface ModalOptions {
  id?: string;
  title: string;
  size?: ModalSize;
  content: ReactNode;
  footer?: ReactNode;
  closeOnBackdrop?: boolean;
  closeOnEsc?: boolean;
  onClose?: () => void;
}

export interface ModalEntry extends Required<Pick<ModalOptions, "id" | "title" | "size" | "closeOnBackdrop" | "closeOnEsc">> {
  content: ReactNode;
  footer?: ReactNode;
  onClose?: () => void;
}

interface ModalState {
  stack: ModalEntry[];
  open: (options: ModalOptions) => string;
  close: (id?: string) => void;
  closeAll: () => void;
}

let counter = 0;

export const useModalStore = create<ModalState>()((set, get) => ({
  stack: [],

  open: (options) => {
    const id = options.id ?? `modal-${++counter}`;
    const entry: ModalEntry = {
      id,
      title: options.title,
      size: options.size ?? "md",
      content: options.content,
      footer: options.footer,
      closeOnBackdrop: options.closeOnBackdrop ?? true,
      closeOnEsc: options.closeOnEsc ?? true,
      onClose: options.onClose,
    };
    set((state) => ({ stack: [...state.stack, entry] }));
    document.body.style.overflow = "hidden";
    return id;
  },

  close: (id) => {
    const { stack } = get();
    if (stack.length === 0) return;
    const targetId = id ?? stack[stack.length - 1].id;
    const target = stack.find((m) => m.id === targetId);
    const next = stack.filter((m) => m.id !== targetId);
    set({ stack: next });
    target?.onClose?.();
    if (next.length === 0) {
      document.body.style.overflow = "";
    }
  },

  closeAll: () => {
    const { stack } = get();
    stack.forEach((m) => m.onClose?.());
    set({ stack: [] });
    document.body.style.overflow = "";
  },
}));
