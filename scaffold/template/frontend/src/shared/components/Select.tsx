import { forwardRef, useId, type SelectHTMLAttributes } from "react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, "children"> {
  options: SelectOption[];
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  placeholder?: string;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ options, label, error, errors, notes, required, placeholder, className, value, defaultValue, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;
    const isPlaceholder = value === "" || (value === undefined && defaultValue === undefined);

    return (
      <div className="rf-field">
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <select
          ref={ref}
          id={id}
          required={required}
          value={value}
          defaultValue={defaultValue}
          className={`rf-select ${hasFieldError(error, errors) ? "rf-select-error" : ""} ${isPlaceholder ? "rf-select-placeholder" : ""} ${className ?? ""}`}
          {...rest}
        >
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options.map((opt) => (
            <option key={opt.value} value={opt.value} disabled={opt.disabled}>
              {opt.label}
            </option>
          ))}
        </select>
        <FieldErrors error={error} errors={errors} />
        {notes && !hasFieldError(error, errors) && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

Select.displayName = "Select";
