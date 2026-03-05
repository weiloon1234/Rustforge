import { createElement, forwardRef, type ButtonHTMLAttributes, type ReactNode } from "react";
import { Loader2 } from "lucide-react";

export type ButtonVariant =
  | "primary"
  | "secondary"
  | "danger"
  | "warning"
  | "error"
  | "success"
  | "info"
  | "plain";

export type ButtonSize = "xs" | "sm" | "md" | "lg";

export interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  type?: "button" | "submit" | "reset";
  variant?: ButtonVariant;
  size?: ButtonSize;
  iconOnly?: boolean;
  busy?: boolean;
  showBusySpinner?: boolean;
}

function joinClasses(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

function renderContent(
  children: ReactNode,
  iconOnly: boolean,
  showSpinner: boolean,
): ReactNode {
  if (showSpinner && iconOnly) {
    return <Loader2 size={14} className="rf-btn-spinner" aria-hidden="true" />;
  }

  return (
    <>
      {showSpinner && <Loader2 size={14} className="rf-btn-spinner" aria-hidden="true" />}
      {children}
    </>
  );
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      type = "button",
      variant = "secondary",
      size = "md",
      iconOnly = false,
      busy = false,
      showBusySpinner = true,
      disabled,
      className,
      children,
      ...rest
    },
    ref,
  ) => {
    const isDisabled = Boolean(disabled || busy);
    const shouldShowSpinner = busy && showBusySpinner;

    return createElement(
      "button",
      {
        ...rest,
        ref,
        type,
        disabled: isDisabled,
        "aria-busy": busy ? "true" : undefined,
        className: joinClasses(
          "rf-btn",
          `rf-btn-size-${size}`,
          `rf-btn-variant-${variant}`,
          iconOnly && "rf-btn-icon",
          className,
        ),
      },
      renderContent(children, iconOnly, shouldShowSpinner),
    );
  },
);

Button.displayName = "Button";
