import { useId, useState, type ChangeEvent } from "react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export interface ColorInputProps {
  value?: string;
  onChange?: (value: string) => void;
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  placeholder?: string;
  containerClassName?: string;
  className?: string;
  required?: boolean;
  disabled?: boolean;
  name?: string;
  id?: string;
}

export function ColorInput({
  value = "",
  onChange,
  label,
  error,
  errors,
  notes,
  placeholder = "#ffffff",
  containerClassName,
  className,
  required,
  disabled,
  name,
  id: externalId,
}: ColorInputProps) {
  const autoId = useId();
  const id = externalId ?? autoId;
  const isError = hasFieldError(error, errors);
  const [preview, setPreview] = useState(value || placeholder);

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    onChange?.(v);
    setPreview(v || placeholder);
  };

  const handleColorPickerChange = (e: ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    onChange?.(v);
    setPreview(v);
  };

  return (
    <div className={containerClassName ?? "mb-4"}>
      {label && (
        <label htmlFor={id} className="mb-1.5 block text-sm font-medium text-[#344054]">
          {label}
          {required && <span className="ml-0.5 text-[#d92d20]">*</span>}
        </label>
      )}
      <div className="flex items-center gap-2">
        <input
          type="color"
          value={preview.startsWith("#") ? preview : "#ffffff"}
          onChange={handleColorPickerChange}
          disabled={disabled}
          className="h-10 w-10 shrink-0 cursor-pointer rounded border border-[#d0d5dd] p-0.5"
        />
        <input
          type="text"
          id={id}
          name={name}
          value={value}
          onChange={handleChange}
          placeholder={placeholder}
          disabled={disabled}
          className={[
            "block w-full rounded-lg border px-3 py-2 text-sm shadow-xs outline-none transition-colors",
            isError
              ? "border-[#fda29b] focus:border-[#fda29b] focus:ring-2 focus:ring-[#fda29b]/25"
              : "border-[#d0d5dd] focus:border-[#84caff] focus:ring-2 focus:ring-[#84caff]/25",
            disabled ? "cursor-not-allowed bg-[#f9fafb] text-[#667085]" : "bg-white text-[#101828]",
            className,
          ]
            .filter(Boolean)
            .join(" ")}
        />
      </div>
      {notes && !isError && (
        <p className="mt-1.5 text-sm text-[#667085]">{notes}</p>
      )}
      <FieldErrors error={error} errors={errors} />
    </div>
  );
}
