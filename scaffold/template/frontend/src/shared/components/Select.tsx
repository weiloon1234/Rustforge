import { useMemo, useState, useRef, useEffect, useId } from "react";
import { ChevronDown, X, Loader2 } from "lucide-react";
import type { AxiosInstance } from "axios";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";
import { useDropdown } from "@shared/hooks/useDropdown";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface RemoteSearchConfig {
  api: AxiosInstance;
  url: string;
  mapResponse: (data: unknown) => SelectOption[];
  minChars?: number;
  debounceMs?: number;
}

export interface SelectProps {
  options?: SelectOption[];
  value?: string;
  onChange?: (value: string) => void;
  name?: string;
  id?: string;
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  placeholder?: string;
  containerClassName?: string;
  className?: string;
  required?: boolean;
  disabled?: boolean;
  searchable?: boolean;
  remoteSearch?: RemoteSearchConfig;
  clearable?: boolean;
}

export function Select({
  options: staticOptions = [],
  value = "",
  onChange,
  name,
  id: externalId,
  label,
  error,
  errors,
  notes,
  placeholder = " ",
  containerClassName,
  className,
  required,
  disabled,
  searchable,
  remoteSearch,
  clearable,
}: SelectProps) {
  const autoId = useId();
  const id = externalId ?? autoId;

  const { dropdownOpen, setDropdownOpen, search, setSearch, close, containerRef, searchRef } = useDropdown();
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const [remoteOptions, setRemoteOptions] = useState<SelectOption[]>([]);
  const [remoteLoading, setRemoteLoading] = useState(false);

  const listRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const remoteSearchRef = useRef(remoteSearch);
  remoteSearchRef.current = remoteSearch;

  const isRemote = !!remoteSearch;
  const isSearchable = searchable || isRemote;
  const minChars = remoteSearch?.minChars ?? 2;
  const debounceMs = remoteSearch?.debounceMs ?? 300;

  // Resolve which options to display
  const baseOptions = isRemote ? remoteOptions : staticOptions;

  // Local filtering (only for non-remote searchable selects)
  const filteredOptions = useMemo(() => {
    if (isRemote || !isSearchable || !search.trim()) return baseOptions;
    const q = search.trim().toLowerCase();
    return baseOptions.filter(
      (o) => o.label.toLowerCase().includes(q) || o.value.toLowerCase().includes(q),
    );
  }, [baseOptions, search, isSearchable, isRemote]);

  // Selected option label
  const selectedLabel = useMemo(() => {
    const allOptions = isRemote ? [...staticOptions, ...remoteOptions] : staticOptions;
    return allOptions.find((o) => o.value === value)?.label;
  }, [value, staticOptions, remoteOptions, isRemote]);

  // Scroll highlighted item into view
  useEffect(() => {
    if (highlightedIndex >= 0 && listRef.current) {
      const items = listRef.current.querySelectorAll("[data-select-item]");
      items[highlightedIndex]?.scrollIntoView({ block: "nearest" });
    }
  }, [highlightedIndex]);

  // Remote search with debounce
  useEffect(() => {
    if (!isRemote || !dropdownOpen) return;
    if (search.trim().length < minChars) {
      setRemoteOptions([]);
      return;
    }

    setRemoteLoading(true);
    const timer = setTimeout(() => {
      const rs = remoteSearchRef.current!;
      rs.api
        .get(rs.url, { params: { q: search.trim() } })
        .then((res) => {
          setRemoteOptions(rs.mapResponse(res.data));
        })
        .catch(() => {
          setRemoteOptions([]);
        })
        .finally(() => {
          setRemoteLoading(false);
        });
    }, debounceMs);

    return () => {
      clearTimeout(timer);
      setRemoteLoading(false);
    };
  }, [search, isRemote, dropdownOpen, minChars, debounceMs]);

  const handleSelect = (optionValue: string) => {
    onChange?.(optionValue);
    close();
    setHighlightedIndex(-1);
    requestAnimationFrame(() => triggerRef.current?.focus());
  };

  const handleClear = (e: React.MouseEvent) => {
    e.stopPropagation();
    onChange?.("");
  };

  const openDropdown = () => {
    setHighlightedIndex(-1);
    if (isRemote) setRemoteOptions([]);
    setDropdownOpen(true);
  };

  const toggleDropdown = () => {
    if (disabled) return;
    if (dropdownOpen) {
      close();
    } else {
      openDropdown();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;

    switch (e.key) {
      case "Escape":
        close();
        setHighlightedIndex(-1);
        triggerRef.current?.focus();
        break;
      case "ArrowDown":
        e.preventDefault();
        if (!dropdownOpen) {
          openDropdown();
        } else {
          setHighlightedIndex((prev) => {
            const enabledOptions = filteredOptions.filter((o) => !o.disabled);
            if (enabledOptions.length === 0) return -1;
            const nextIdx = prev + 1;
            return nextIdx >= filteredOptions.length ? 0 : findNextEnabled(filteredOptions, nextIdx, 1);
          });
        }
        break;
      case "ArrowUp":
        e.preventDefault();
        if (dropdownOpen) {
          setHighlightedIndex((prev) => {
            if (prev <= 0) return filteredOptions.length - 1;
            return findNextEnabled(filteredOptions, prev - 1, -1);
          });
        }
        break;
      case "Enter":
        e.preventDefault();
        if (dropdownOpen && highlightedIndex >= 0 && highlightedIndex < filteredOptions.length) {
          const opt = filteredOptions[highlightedIndex];
          if (!opt.disabled) handleSelect(opt.value);
        } else if (!dropdownOpen) {
          openDropdown();
        }
        break;
    }
  };

  const isPlaceholder = !value;
  const hasError = hasFieldError(error, errors);

  return (
    <div className={`rf-field ${containerClassName ?? ""}`} ref={containerRef} onKeyDown={handleKeyDown}>
      {label && (
        <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </label>
      )}

      {/* Hidden input for form serialization */}
      {name && <input type="hidden" name={name} value={value} />}

      <div className="relative">
        {/* Trigger button */}
        <button
          ref={triggerRef}
          id={id}
          type="button"
          onClick={toggleDropdown}
          disabled={disabled}
          className={`rf-select-trigger ${hasError ? "rf-select-error" : ""} ${isPlaceholder ? "rf-select-placeholder" : ""} ${className ?? ""}`}
        >
          <span className="flex-1 truncate text-left">
            {selectedLabel ?? (placeholder !== " " ? placeholder : "\u00A0")}
          </span>
          <span className="flex items-center gap-0.5 shrink-0">
            {clearable && value && !disabled && (
              <span
                role="button"
                tabIndex={-1}
                onClick={handleClear}
                className="p-0.5 rounded hover:bg-surface-hover transition-colors"
              >
                <X className="h-3.5 w-3.5 text-muted" />
              </span>
            )}
            <ChevronDown className={`h-4 w-4 text-muted transition-transform ${dropdownOpen ? "rotate-180" : ""}`} />
          </span>
        </button>

        {/* Dropdown panel */}
        {dropdownOpen && (
          <div className="rf-select-dropdown">
            {isSearchable && (
              <div className="rf-select-dropdown-search-wrapper">
                <input
                  ref={searchRef}
                  type="text"
                  className="rf-select-dropdown-search"
                  placeholder={isRemote ? `Type ${minChars}+ chars to search...` : "Search..."}
                  value={search}
                  onChange={(e) => {
                    setSearch(e.target.value);
                    setHighlightedIndex(-1);
                  }}
                />
              </div>
            )}
            <div ref={listRef} className="rf-select-dropdown-list">
              {remoteLoading && (
                <div className="flex items-center justify-center gap-2 px-3 py-4 text-sm text-muted">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span>Searching...</span>
                </div>
              )}
              {!remoteLoading && filteredOptions.length === 0 && (
                <div className="px-3 py-4 text-sm text-muted text-center">
                  {isRemote && search.trim().length < minChars
                    ? `Type ${minChars}+ characters to search`
                    : "No options found"}
                </div>
              )}
              {!remoteLoading &&
                filteredOptions.map((opt, idx) => (
                  <button
                    key={opt.value}
                    type="button"
                    data-select-item
                    disabled={opt.disabled}
                    className={`rf-select-dropdown-item ${
                      opt.value === value ? "rf-select-dropdown-item-active" : ""
                    } ${highlightedIndex === idx ? "bg-surface-hover" : ""} ${
                      opt.disabled ? "opacity-50 cursor-not-allowed" : ""
                    }`}
                    onClick={() => !opt.disabled && handleSelect(opt.value)}
                    onMouseEnter={() => setHighlightedIndex(idx)}
                  >
                    {opt.label}
                  </button>
                ))}
            </div>
          </div>
        )}
      </div>

      <FieldErrors error={error} errors={errors} />
      {notes && !hasError && <p className="rf-note">{notes}</p>}
    </div>
  );
}

function findNextEnabled(options: SelectOption[], startIdx: number, direction: 1 | -1): number {
  let idx = startIdx;
  const len = options.length;
  for (let i = 0; i < len; i++) {
    if (idx < 0) idx = len - 1;
    if (idx >= len) idx = 0;
    if (!options[idx].disabled) return idx;
    idx += direction;
  }
  return -1;
}

Select.displayName = "Select";
