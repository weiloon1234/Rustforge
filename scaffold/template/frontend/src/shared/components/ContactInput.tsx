import type { ChangeEvent } from "react";
import { useMemo, useState, useRef, useEffect, useCallback } from "react";
import { ChevronDown } from "lucide-react";
import { useTranslation } from "react-i18next";
import { availableCountries, defaultCountryIso2, findCountryByIso2 } from "@shared/countryRuntime";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";
import type { CountryRuntime } from "@shared/types/platform";

export interface ContactInputValue {
  country_iso2: string;
  phone_number: string;
}

export interface ContactInputProps {
  value: ContactInputValue;
  onChange: (value: ContactInputValue) => void;
  countries?: CountryRuntime[];
  label?: string;
  countryName?: string;
  phoneName?: string;
  disabled?: boolean;
  required?: boolean;
  countryRequired?: boolean;
  phoneRequired?: boolean;
  countryError?: string;
  countryErrors?: string[];
  phoneError?: string;
  phoneErrors?: string[];
  containerClassName?: string;
}

function formatCallingCode(country: CountryRuntime): string {
  return country.calling_code?.trim() ?? "";
}

function formatFlag(country: CountryRuntime): string {
  return country.flag_emoji?.trim() ?? "";
}

export function ContactInput({
  value,
  onChange,
  countries,
  label,
  countryName: _countryName,
  phoneName: _phoneName,
  disabled = false,
  required = false,
  countryRequired,
  phoneRequired,
  countryError,
  countryErrors,
  phoneError,
  phoneErrors,
  containerClassName,
}: ContactInputProps) {
  const { t } = useTranslation();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [search, setSearch] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const phoneRef = useRef<HTMLInputElement>(null);

  const resolvedCountries = useMemo(
    () => countries ?? availableCountries(),
    [countries],
  );

  const fallbackIso2 = defaultCountryIso2(resolvedCountries) ?? "";
  const selectedIso2 = (value.country_iso2 || fallbackIso2).toUpperCase();
  const selectedCountry = useMemo(
    () => findCountryByIso2(selectedIso2, resolvedCountries),
    [selectedIso2, resolvedCountries],
  );

  const filteredCountries = useMemo(() => {
    if (!search.trim()) return resolvedCountries;
    const q = search.trim().toLowerCase();
    return resolvedCountries.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.iso2.toLowerCase().includes(q) ||
        (c.calling_code?.includes(q) ?? false),
    );
  }, [resolvedCountries, search]);

  const hasError = hasFieldError(countryError, countryErrors) || hasFieldError(phoneError, phoneErrors);

  // Close dropdown on outside click
  const handleClickOutside = useCallback((e: MouseEvent) => {
    if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
      setDropdownOpen(false);
      setSearch("");
    }
  }, []);

  useEffect(() => {
    if (dropdownOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      // Focus search input when dropdown opens
      requestAnimationFrame(() => searchRef.current?.focus());
    } else {
      document.removeEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [dropdownOpen, handleClickOutside]);

  // Scroll active item into view when dropdown opens
  useEffect(() => {
    if (dropdownOpen && dropdownRef.current) {
      const active = dropdownRef.current.querySelector("[data-active]");
      if (active) {
        active.scrollIntoView({ block: "nearest" });
      }
    }
  }, [dropdownOpen]);

  const handleSelectCountry = (iso2: string) => {
    onChange({
      country_iso2: iso2.toUpperCase(),
      phone_number: value.phone_number,
    });
    setDropdownOpen(false);
    setSearch("");
    // Focus phone input after selection
    requestAnimationFrame(() => phoneRef.current?.focus());
  };

  const handlePhoneChange = (event: ChangeEvent<HTMLInputElement>) => {
    onChange({
      country_iso2: selectedIso2,
      phone_number: event.target.value,
    });
  };

  const toggleDropdown = () => {
    if (disabled || resolvedCountries.length === 0) return;
    setDropdownOpen((prev) => !prev);
    if (dropdownOpen) setSearch("");
  };

  const isRequired = required || countryRequired || phoneRequired;
  const callingCode = selectedCountry ? formatCallingCode(selectedCountry) : "";
  const flag = selectedCountry ? formatFlag(selectedCountry) : "";

  return (
    <div className={`rf-field ${containerClassName ?? ""}`} ref={containerRef}>
      {label !== undefined && label !== null ? (
        <label className={`rf-label ${isRequired ? "rf-label-required" : ""}`}>
          {label}
        </label>
      ) : (
        <label className={`rf-label ${isRequired ? "rf-label-required" : ""}`}>
          {t("Phone Number")}
        </label>
      )}

      <div className="relative">
        {/* Joined input container */}
        <div
          className={`rf-contact-input-group ${hasError ? "rf-contact-input-group-error" : ""} ${disabled ? "rf-contact-input-group-disabled" : ""}`}
        >
          {/* Country selector button */}
          <button
            type="button"
            onClick={toggleDropdown}
            disabled={disabled || resolvedCountries.length === 0}
            className="rf-contact-country-trigger"
            tabIndex={-1}
          >
            {flag && <span className="text-base leading-none">{flag}</span>}
            {callingCode && (
              <span className="text-sm text-muted whitespace-nowrap">{callingCode}</span>
            )}
            <ChevronDown
              size={14}
              className={`text-muted transition-transform ${dropdownOpen ? "rotate-180" : ""}`}
            />
          </button>

          {/* Divider */}
          <div className="rf-contact-divider" />

          {/* Phone number input */}
          <input
            ref={phoneRef}
            type="tel"
            inputMode="tel"
            value={value.phone_number}
            onChange={handlePhoneChange}
            placeholder={t("Enter phone number")}
            disabled={disabled}
            required={phoneRequired ?? required}
            className="rf-contact-phone-input"
          />
        </div>

        {/* Dropdown */}
        {dropdownOpen && (
          <div className="rf-contact-dropdown" ref={dropdownRef}>
            {/* Search */}
            <div className="rf-contact-dropdown-search-wrapper">
              <input
                ref={searchRef}
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder={t("Search country")}
                className="rf-contact-dropdown-search"
                onKeyDown={(e) => {
                  if (e.key === "Escape") {
                    setDropdownOpen(false);
                    setSearch("");
                  }
                }}
              />
            </div>
            {/* Options */}
            <div className="rf-contact-dropdown-list">
              {filteredCountries.length === 0 ? (
                <div className="px-3 py-2 text-sm text-muted">
                  {t("No countries available")}
                </div>
              ) : (
                filteredCountries.map((country) => {
                  const active = country.iso2.toUpperCase() === selectedIso2;
                  const cFlag = formatFlag(country);
                  const cCode = formatCallingCode(country);
                  return (
                    <button
                      key={country.iso2}
                      type="button"
                      onClick={() => handleSelectCountry(country.iso2)}
                      className={`rf-contact-dropdown-item ${active ? "rf-contact-dropdown-item-active" : ""}`}
                      {...(active ? { "data-active": true } : {})}
                    >
                      {cFlag && <span className="text-base leading-none">{cFlag}</span>}
                      <span className="flex-1 truncate text-left text-sm">
                        {country.name}
                      </span>
                      {cCode && (
                        <span className="text-xs text-muted">{cCode}</span>
                      )}
                    </button>
                  );
                })
              )}
            </div>
          </div>
        )}
      </div>

      <FieldErrors error={countryError} errors={[...(countryErrors ?? []), ...(phoneErrors ?? [])]} />
      {!hasError && phoneError && (
        <FieldErrors error={phoneError} />
      )}
    </div>
  );
}

// Re-export SearchCountry for standalone usage
export function SearchCountryInput({
  value,
  onChange,
  countries,
  label,
  disabled = false,
  required = false,
  error,
  errors,
  containerClassName,
}: {
  value: string;
  onChange: (iso2: string) => void;
  countries?: CountryRuntime[];
  label?: string;
  disabled?: boolean;
  required?: boolean;
  error?: string;
  errors?: string[];
  containerClassName?: string;
}) {
  const { t } = useTranslation();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [search, setSearch] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const resolvedCountries = useMemo(
    () => countries ?? availableCountries(),
    [countries],
  );

  const selectedCountry = useMemo(
    () => findCountryByIso2(value, resolvedCountries),
    [value, resolvedCountries],
  );

  const filteredCountries = useMemo(() => {
    if (!search.trim()) return resolvedCountries;
    const q = search.trim().toLowerCase();
    return resolvedCountries.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.iso2.toLowerCase().includes(q),
    );
  }, [resolvedCountries, search]);

  const handleClickOutside = useCallback((e: MouseEvent) => {
    if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
      setDropdownOpen(false);
      setSearch("");
    }
  }, []);

  useEffect(() => {
    if (dropdownOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      requestAnimationFrame(() => searchRef.current?.focus());
    } else {
      document.removeEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [dropdownOpen, handleClickOutside]);

  return (
    <div className={`rf-field ${containerClassName ?? ""}`} ref={containerRef}>
      {label && (
        <label className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </label>
      )}
      <div className="relative">
        <button
          type="button"
          onClick={() => !disabled && setDropdownOpen((p) => !p)}
          disabled={disabled}
          className={`rf-input flex items-center gap-2 text-left ${hasFieldError(error, errors) ? "rf-input-error" : ""}`}
        >
          {selectedCountry ? (
            <>
              {formatFlag(selectedCountry) && (
                <span className="text-base">{formatFlag(selectedCountry)}</span>
              )}
              <span className="flex-1 truncate">{selectedCountry.name} ({selectedCountry.iso2})</span>
            </>
          ) : (
            <span className="flex-1 text-input-placeholder">{t("Select country")}</span>
          )}
          <ChevronDown size={14} className="text-muted" />
        </button>

        {dropdownOpen && (
          <div className="rf-contact-dropdown">
            <div className="rf-contact-dropdown-search-wrapper">
              <input
                ref={searchRef}
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder={t("Search country")}
                className="rf-contact-dropdown-search"
                onKeyDown={(e) => {
                  if (e.key === "Escape") {
                    setDropdownOpen(false);
                    setSearch("");
                  }
                }}
              />
            </div>
            <div className="rf-contact-dropdown-list">
              {filteredCountries.map((country) => {
                const active = country.iso2.toUpperCase() === value.toUpperCase();
                return (
                  <button
                    key={country.iso2}
                    type="button"
                    onClick={() => {
                      onChange(country.iso2);
                      setDropdownOpen(false);
                      setSearch("");
                    }}
                    className={`rf-contact-dropdown-item ${active ? "rf-contact-dropdown-item-active" : ""}`}
                  >
                    {formatFlag(country) && (
                      <span className="text-base">{formatFlag(country)}</span>
                    )}
                    <span className="flex-1 truncate text-left text-sm">{country.name} ({country.iso2})</span>
                  </button>
                );
              })}
            </div>
          </div>
        )}
      </div>
      <FieldErrors error={error} errors={errors} />
    </div>
  );
}
