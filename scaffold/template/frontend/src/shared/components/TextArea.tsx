import { forwardRef, useId, type TextareaHTMLAttributes } from "react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
}

export const TextArea = forwardRef<HTMLTextAreaElement, TextAreaProps>(
  ({ label, error, errors, notes, required, className, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;

    return (
      <div className="rf-field">
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <textarea
          ref={ref}
          id={id}
          required={required}
          className={`rf-textarea ${hasFieldError(error, errors) ? "rf-textarea-error" : ""} ${className ?? ""}`}
          {...rest}
        />
        <FieldErrors error={error} errors={errors} />
        {notes && !hasFieldError(error, errors) && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

TextArea.displayName = "TextArea";
