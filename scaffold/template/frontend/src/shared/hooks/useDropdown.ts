import { useState, useRef, useEffect, useCallback, type RefObject } from "react";

export interface UseDropdownOptions {
  onClose?: () => void;
}

export interface UseDropdownReturn {
  dropdownOpen: boolean;
  setDropdownOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  search: string;
  setSearch: (search: string) => void;
  open: () => void;
  close: () => void;
  toggle: () => void;
  containerRef: RefObject<HTMLDivElement | null>;
  searchRef: RefObject<HTMLInputElement | null>;
}

/**
 * Shared dropdown state and behavior for custom dropdown components.
 * Handles click-outside detection, search state, and focus management.
 */
export function useDropdown(options?: UseDropdownOptions): UseDropdownReturn {
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [search, setSearch] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const close = useCallback(() => {
    setDropdownOpen(false);
    setSearch("");
    options?.onClose?.();
  }, [options?.onClose]);

  const open = useCallback(() => {
    setDropdownOpen(true);
  }, []);

  const toggle = useCallback(() => {
    setDropdownOpen((prev) => {
      if (prev) {
        setSearch("");
        options?.onClose?.();
      }
      return !prev;
    });
  }, [options?.onClose]);

  // Click outside to close
  const handleClickOutside = useCallback(
    (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        close();
      }
    },
    [close],
  );

  useEffect(() => {
    if (!dropdownOpen) return;
    document.addEventListener("mousedown", handleClickOutside);
    requestAnimationFrame(() => searchRef.current?.focus());
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [dropdownOpen, handleClickOutside]);

  return {
    dropdownOpen,
    setDropdownOpen,
    search,
    setSearch,
    open,
    close,
    toggle,
    containerRef,
    searchRef,
  };
}
