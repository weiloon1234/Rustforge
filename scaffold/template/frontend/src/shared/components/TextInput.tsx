import { forwardRef, useId, useState, type InputHTMLAttributes } from "react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

type InputType = "text" | "email" | "password" | "search" | "url" | "tel" | "number" | "money" | "pin";

export interface TextInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  type?: InputType;
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
}

function formatMoney(value: string): string {
  const num = value.replace(/[^0-9.]/g, "");
  const parts = num.split(".");
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  if (parts.length > 2) parts.length = 2;
  if (parts[1] !== undefined) parts[1] = parts[1].slice(0, 2);
  return parts.join(".");
}

function rawMoney(display: string): string {
  return display.replace(/,/g, "");
}

export const TextInput = forwardRef<HTMLInputElement, TextInputProps>(
  ({ type = "text", label, error, errors, notes, required, className, onChange, value, defaultValue, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;
    const isMoney = type === "money";
    const isPin = type === "pin";

    const [moneyDisplay, setMoneyDisplay] = useState(() => {
      const init = (value ?? defaultValue ?? "") as string;
      return isMoney ? formatMoney(init) : "";
    });

    const resolvedType = isMoney ? "text" : isPin ? "password" : type;

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      if (isMoney) {
        const formatted = formatMoney(e.target.value);
        setMoneyDisplay(formatted);
        const synth = { ...e, target: { ...e.target, value: rawMoney(formatted) } } as React.ChangeEvent<HTMLInputElement>;
        onChange?.(synth);
      } else if (isPin) {
        e.target.value = e.target.value.replace(/\D/g, "");
        onChange?.(e);
      } else {
        onChange?.(e);
      }
    };

    const inputMode = isMoney ? "decimal" as const : isPin ? "numeric" as const : undefined;

    return (
      <div className="rf-field">
        {label && (
          <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={id}
          type={resolvedType}
          inputMode={inputMode}
          required={required}
          className={`rf-input ${hasFieldError(error, errors) ? "rf-input-error" : ""} ${className ?? ""}`}
          onChange={handleChange}
          value={isMoney ? moneyDisplay : value}
          defaultValue={isMoney ? undefined : defaultValue}
          {...rest}
        />
        <FieldErrors error={error} errors={errors} />
        {notes && !hasFieldError(error, errors) && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

TextInput.displayName = "TextInput";
