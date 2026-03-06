import { useId } from "react";
import { Checkbox } from "@shared/components/Checkbox";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export interface CheckboxGroupOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface CheckboxGroupProps {
  name: string;
  options: CheckboxGroupOption[];
  value?: string[];
  onChange?: (value: string[]) => void;
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  required?: boolean;
  disabled?: boolean;
  columns?: number;
}

export function CheckboxGroup({ options, value = [], onChange, label, error, errors, notes, required, disabled, columns = 2 }: CheckboxGroupProps) {
  const groupId = useId();
  const hasError = hasFieldError(error, errors);

  return (
    <div className="rf-field">
      {label && (
        <span className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </span>
      )}
      <div
        role="group"
        aria-labelledby={label ? `${groupId}-label` : undefined}
        className="rf-checkbox-group"
        style={{ gridTemplateColumns: `repeat(${columns}, 1fr)` }}
      >
        {options.map((opt) => {
          const isChecked = value.includes(opt.value);
          return (
            <Checkbox
              key={opt.value}
              label={opt.label}
              checked={isChecked}
              onChange={(e) => {
                if (!onChange) return;
                if (e.target.checked) {
                  onChange([...value, opt.value]);
                } else {
                  onChange(value.filter((v) => v !== opt.value));
                }
              }}
              disabled={disabled || opt.disabled}
              containerClassName="!mb-0"
            />
          );
        })}
      </div>
      <FieldErrors error={error} errors={errors} />
      {notes && !hasError && <p className="rf-note">{notes}</p>}
    </div>
  );
}
