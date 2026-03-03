import { forwardRef, useId, useState, type InputHTMLAttributes } from "react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

type InputType =
  | "text"
  | "email"
  | "password"
  | "search"
  | "url"
  | "tel"
  | "number"
  | "money"
  | "atm"
  | "pin"
  | "date"
  | "datetime-local"
  | "time";

export interface TextInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  type?: InputType;
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  containerClassName?: string;
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

function formatAtm(value: string): string {
  const digits = value.replace(/\D/g, "");
  if (!digits) return "";
  const padded = digits.padStart(3, "0");
  const integer = padded.slice(0, -2).replace(/^0+(?=\d)/, "") || "0";
  const cents = padded.slice(-2);
  const grouped = integer.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  return `${grouped}.${cents}`;
}

function rawAtm(display: string): string {
  return display.replace(/,/g, "");
}

export const TextInput = forwardRef<HTMLInputElement, TextInputProps>(
  ({ type = "text", label, error, errors, notes, required, className, containerClassName, onChange, value, defaultValue, id: externalId, ...rest }, ref) => {
    const autoId = useId();
    const id = externalId ?? autoId;
    const isMoney = type === "money";
    const isAtm = type === "atm";
    const isPin = type === "pin";
    const isFormattedInput = isMoney || isAtm;

    const [displayValue, setDisplayValue] = useState(() => {
      const init = (value ?? defaultValue ?? "") as string;
      if (isMoney) return formatMoney(init);
      if (isAtm) return formatAtm(init);
      return "";
    });

    const resolvedType = isFormattedInput ? "text" : isPin ? "password" : type;

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      if (isMoney) {
        const formatted = formatMoney(e.target.value);
        setDisplayValue(formatted);
        const synth = { ...e, target: { ...e.target, value: rawMoney(formatted) } } as React.ChangeEvent<HTMLInputElement>;
        onChange?.(synth);
      } else if (isAtm) {
        const formatted = formatAtm(e.target.value);
        setDisplayValue(formatted);
        const synth = { ...e, target: { ...e.target, value: rawAtm(formatted) } } as React.ChangeEvent<HTMLInputElement>;
        onChange?.(synth);
      } else if (isPin) {
        e.target.value = e.target.value.replace(/\D/g, "");
        onChange?.(e);
      } else {
        onChange?.(e);
      }
    };

    const inputMode = isMoney
      ? ("decimal" as const)
      : isAtm || isPin
        ? ("numeric" as const)
        : undefined;

    return (
      <div className={`rf-field ${containerClassName ?? ""}`}>
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
          value={isFormattedInput ? displayValue : value}
          defaultValue={isFormattedInput ? undefined : defaultValue}
          {...rest}
        />
        <FieldErrors error={error} errors={errors} />
        {notes && !hasFieldError(error, errors) && <p className="rf-note">{notes}</p>}
      </div>
    );
  },
);

TextInput.displayName = "TextInput";
