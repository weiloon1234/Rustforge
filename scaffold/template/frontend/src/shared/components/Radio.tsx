import { useId } from "react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export interface RadioOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface RadioProps {
  name: string;
  options: RadioOption[];
  value?: string;
  onChange?: (value: string) => void;
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  required?: boolean;
  disabled?: boolean;
  className?: string;
}

export function Radio({ name, options, value, onChange, label, error, errors, notes, required, disabled, className }: RadioProps) {
  const groupId = useId();
  const hasError = hasFieldError(error, errors);

  return (
    <div className="rf-field">
      {label && (
        <span className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </span>
      )}
      <div role="radiogroup" aria-labelledby={label ? `${groupId}-label` : undefined} className={`rf-radio-group ${className ?? ""}`}>
        {options.map((opt) => {
          const optId = `${groupId}-${opt.value}`;
          const isSelected = value === opt.value;
          return (
            <label key={opt.value} htmlFor={optId} className="rf-radio-wrapper">
              <input
                id={optId}
                type="radio"
                name={name}
                value={opt.value}
                checked={isSelected}
                onChange={() => onChange?.(opt.value)}
                disabled={disabled || opt.disabled}
                className="rf-radio-hidden"
              />
              <span className={`rf-radio-circle ${hasError ? "rf-radio-error" : ""}`}>
                {isSelected && <span className="rf-radio-dot" />}
              </span>
              <span className="rf-radio-label">{opt.label}</span>
            </label>
          );
        })}
      </div>
      <FieldErrors error={error} errors={errors} />
      {notes && !hasError && <p className="rf-note">{notes}</p>}
    </div>
  );
}
