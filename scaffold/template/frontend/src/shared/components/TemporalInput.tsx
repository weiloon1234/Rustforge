import {
  forwardRef,
  useId,
  useMemo,
  type FocusEvent,
  type FocusEventHandler,
  type KeyboardEvent,
  type KeyboardEventHandler,
  type SyntheticEvent,
  type ChangeEvent,
  type InputHTMLAttributes,
} from "react";
import DatePicker from "react-datepicker";
import "react-datepicker/dist/react-datepicker.css";
import { CalendarDays, Clock3 } from "lucide-react";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

export type TemporalInputType = "date" | "datetime-local" | "time";

export interface TemporalInputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label?: string;
  error?: string;
  errors?: string[];
  notes?: string;
  containerClassName?: string;
}

interface PickerInputProps extends InputHTMLAttributes<HTMLInputElement> {
  pickerType: TemporalInputType;
  hasError: boolean;
}

const PickerTextInput = forwardRef<HTMLInputElement, PickerInputProps>(
  ({ pickerType, hasError, className, ...rest }, ref) => {
    const Icon = pickerType === "time" ? Clock3 : CalendarDays;
    return (
      <div className="rf-picker">
        <input
          ref={ref}
          className={`rf-input rf-picker-input ${hasError ? "rf-input-error" : ""} ${className ?? ""}`}
          {...rest}
        />
        <span className="rf-picker-icon" aria-hidden="true">
          <Icon size={16} />
        </span>
      </div>
    );
  },
);

PickerTextInput.displayName = "PickerTextInput";

function pad2(value: number): string {
  return String(value).padStart(2, "0");
}

function formatTemporalValue(type: TemporalInputType, date: Date | null): string {
  if (!date) return "";
  const year = date.getFullYear();
  const month = pad2(date.getMonth() + 1);
  const day = pad2(date.getDate());
  const hour = pad2(date.getHours());
  const minute = pad2(date.getMinutes());

  if (type === "date") return `${year}-${month}-${day}`;
  if (type === "time") return `${hour}:${minute}`;
  return `${year}-${month}-${day}T${hour}:${minute}`;
}

function parseTemporalValue(type: TemporalInputType, value: string | number | readonly string[] | undefined): Date | null {
  if (typeof value !== "string") return null;
  const raw = value.trim();
  if (!raw) return null;

  if (type === "date") {
    const matched = raw.match(/^(\d{4})-(\d{2})-(\d{2})$/);
    if (!matched) return null;
    const year = Number(matched[1]);
    const month = Number(matched[2]) - 1;
    const day = Number(matched[3]);
    return new Date(year, month, day, 0, 0, 0, 0);
  }

  if (type === "time") {
    const matched = raw.match(/^(\d{2}):(\d{2})$/);
    if (!matched) return null;
    const now = new Date();
    now.setHours(Number(matched[1]), Number(matched[2]), 0, 0);
    return now;
  }

  const matched = raw.match(
    /^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})(?::(\d{2}))?$/,
  );
  if (!matched) return null;
  const year = Number(matched[1]);
  const month = Number(matched[2]) - 1;
  const day = Number(matched[3]);
  const hour = Number(matched[4]);
  const minute = Number(matched[5]);
  const second = Number(matched[6] ?? "0");
  return new Date(year, month, day, hour, minute, second, 0);
}

function normalizePickerDate(value: Date | null | [Date | null, Date | null]): Date | null {
  if (Array.isArray(value)) return value[0] ?? null;
  return value;
}

function TemporalInput({
  pickerType,
  label,
  error,
  errors,
  notes,
  required,
  className,
  containerClassName,
  id: externalId,
  value,
  onChange,
  placeholder,
  disabled,
  name,
  onBlur,
  onFocus,
  onKeyDown,
}: TemporalInputProps & { pickerType: TemporalInputType }) {
  const autoId = useId();
  const id = externalId ?? autoId;
  const hasError = hasFieldError(error, errors);
  const selected = useMemo(() => parseTemporalValue(pickerType, value), [pickerType, value]);

  const emitChange = (nextValue: string) => {
    if (!onChange) return;
    const event = {
      target: { value: nextValue },
      currentTarget: { value: nextValue },
    } as unknown as ChangeEvent<HTMLInputElement>;
    onChange(event);
  };

  const handlePickerKeyDown: ((event: KeyboardEvent<HTMLElement>) => void) | undefined = onKeyDown
    ? (event) => {
        (onKeyDown as KeyboardEventHandler<HTMLInputElement>)(
          event as unknown as KeyboardEvent<HTMLInputElement>,
        );
      }
    : undefined;
  const handlePickerBlur: FocusEventHandler<HTMLElement> | undefined = onBlur
    ? (event) => {
        onBlur(event as unknown as FocusEvent<HTMLInputElement>);
      }
    : undefined;
  const handlePickerFocus: FocusEventHandler<HTMLElement> | undefined = onFocus
    ? (event) => {
        onFocus(event as unknown as FocusEvent<HTMLInputElement>);
      }
    : undefined;

  return (
    <div className={`rf-field ${containerClassName ?? ""}`}>
      {label && (
        <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </label>
      )}
      <DatePicker
        id={id}
        selected={selected}
        onChange={(date: Date | null | [Date | null, Date | null]) =>
          emitChange(formatTemporalValue(pickerType, normalizePickerDate(date)))
        }
        onChangeRaw={(event?: SyntheticEvent<HTMLElement>) => {
          if (!event) return;
          const target = event.target;
          if (!(target instanceof HTMLInputElement)) return;
          emitChange(target.value);
        }}
        dateFormat={
          pickerType === "date"
            ? "yyyy-MM-dd"
            : pickerType === "time"
              ? "HH:mm"
              : "yyyy-MM-dd HH:mm"
        }
        showTimeSelect={pickerType === "datetime-local" || pickerType === "time"}
        showTimeSelectOnly={pickerType === "time"}
        timeIntervals={5}
        timeCaption={pickerType === "time" ? "Time" : "Time"}
        placeholderText={placeholder}
        disabled={disabled}
        name={name}
        onBlur={handlePickerBlur}
        onFocus={handlePickerFocus}
        onKeyDown={handlePickerKeyDown}
        required={required}
        autoComplete="off"
        popperPlacement="bottom-start"
        popperClassName="rf-picker-popper"
        popperProps={{ strategy: "fixed" }}
        calendarClassName="rf-picker-calendar"
        customInput={
          <PickerTextInput
            pickerType={pickerType}
            hasError={hasError}
            className={className}
          />
        }
      />
      <FieldErrors error={error} errors={errors} />
      {notes && !hasError && <p className="rf-note">{notes}</p>}
    </div>
  );
}

export function DatePickerInput(props: TemporalInputProps) {
  return <TemporalInput pickerType="date" {...props} />;
}

export function DateTimePickerInput(props: TemporalInputProps) {
  return <TemporalInput pickerType="datetime-local" {...props} />;
}

export function TimePickerInput(props: TemporalInputProps) {
  return <TemporalInput pickerType="time" {...props} />;
}
