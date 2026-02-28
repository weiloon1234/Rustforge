import { forwardRef, useId, type InputHTMLAttributes } from "react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(
  ({ label, error, errors, notes, className, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;
    const hasError = hasFieldError(error, errors);

    return (
      <div className="rf-field">
        <label htmlFor={id} className="rf-checkbox-wrapper">
          <input
            ref={ref}
            id={id}
            type="checkbox"
            className={`rf-checkbox-hidden ${className ?? ""}`}
            {...rest}
          />
          <span className={`rf-checkbox-box ${hasError ? "rf-checkbox-error" : ""}`}>
            <svg className="rf-checkbox-icon" viewBox="0 0 12 12" fill="none">
              <path d="M2.5 6L5 8.5L9.5 3.5" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </span>
          {label && <span className="rf-checkbox-label">{label}</span>}
        </label>
        <FieldErrors error={error} errors={errors} />
        {notes && !hasError && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

Checkbox.displayName = "Checkbox";
