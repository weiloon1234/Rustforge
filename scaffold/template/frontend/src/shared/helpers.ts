import Swal, { type SweetAlertResult } from "sweetalert2";

// ── Alert types ──────────────────────────────────────

interface AlertOptions {
  title?: string;
  message: string;
  callback?: (result: SweetAlertResult) => void | Promise<void>;
}

interface AlertConfirmOptions extends AlertOptions {
  confirmText?: string;
  cancelText?: string;
}

// ── Alert wrappers ───────────────────────────────────
// Wrapped so the underlying library can be swapped without
// touching every call-site in the application.

export async function alertConfirm(options: AlertConfirmOptions): Promise<void> {
  const result = await Swal.fire({
    title: options.title ?? "Are you sure?",
    text: options.message,
    icon: "question",
    showCancelButton: true,
    confirmButtonText: options.confirmText ?? "Confirm",
    cancelButtonText: options.cancelText ?? "Cancel",
    reverseButtons: true,
  });
  await options.callback?.(result);
}

export async function alertSuccess(options: AlertOptions): Promise<void> {
  const result = await Swal.fire({
    title: options.title ?? "Success",
    text: options.message,
    icon: "success",
  });
  await options.callback?.(result);
}

export async function alertError(options: AlertOptions): Promise<void> {
  const result = await Swal.fire({
    title: options.title ?? "Error",
    text: options.message,
    icon: "error",
  });
  await options.callback?.(result);
}

export async function alertWarning(options: AlertOptions): Promise<void> {
  const result = await Swal.fire({
    title: options.title ?? "Warning",
    text: options.message,
    icon: "warning",
  });
  await options.callback?.(result);
}

export async function alertInfo(options: AlertOptions): Promise<void> {
  const result = await Swal.fire({
    title: options.title ?? "Info",
    text: options.message,
    icon: "info",
  });
  await options.callback?.(result);
}

// ── Formatting helpers ───────────────────────────────

export function moneyFormat(value: number, decimals: number = 2): string {
  return value.toLocaleString(undefined, {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

/**
 * Converts a UTC/server timestamp string to the browser's local timezone.
 *
 * @param value - ISO 8601 / RFC 3339 date string from the backend (e.g. "2025-03-01T12:00:00Z")
 * @param format - Output format string. Tokens: Y (year), m (month 01), d (day 01),
 *                 H (24h hour), h (12h hour), i (minute), s (second), A (AM/PM).
 *                 Default: "Y-m-d h:i:s A"
 * @returns Formatted local date string, or "—" if the value is empty/invalid.
 */
export function formatDateTime(value: string | null | undefined, format: string = "Y-m-d h:i:s A"): string {
  if (!value) return "—";
  const date = new Date(value);
  if (isNaN(date.getTime())) return "—";

  const pad = (n: number) => String(n).padStart(2, "0");
  const hours24 = date.getHours();
  const hours12 = hours24 % 12 || 12;

  const tokens: Record<string, string> = {
    Y: String(date.getFullYear()),
    m: pad(date.getMonth() + 1),
    d: pad(date.getDate()),
    H: pad(hours24),
    h: pad(hours12),
    i: pad(date.getMinutes()),
    s: pad(date.getSeconds()),
    A: hours24 >= 12 ? "PM" : "AM",
  };

  return format.replace(/Y|m|d|H|h|i|s|A/g, (tok) => tokens[tok] ?? tok);
}
