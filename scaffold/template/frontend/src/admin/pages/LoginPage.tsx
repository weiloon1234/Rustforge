import { useMemo, useState } from "react";
import { Check, Shield } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useAutoForm } from "@shared/useAutoForm";
import { Button } from "@shared/components";
import { useAuthStore } from "@admin/stores/auth";
import { api } from "@admin/api";
import type { AdminAuthOutput } from "@admin/types";
import { useLocaleStore } from "@shared/components";
import type { LocaleCode } from "@shared/types/platform";

function LoginLocaleSelector() {
  const { t } = useTranslation();
  const locale = useLocaleStore((s) => s.locale);
  const defaultLocale = useLocaleStore((s) => s.defaultLocale);
  const availableLocales = useLocaleStore((s) => s.availableLocales);
  const setLocale = useLocaleStore((s) => s.setLocale);
  const [busyLocale, setBusyLocale] = useState<LocaleCode | null>(null);

  const localeOptions = useMemo<LocaleCode[]>(() => {
    if (availableLocales.length > 0) return availableLocales;
    return [defaultLocale];
  }, [availableLocales, defaultLocale]);

  const handleLocaleChange = async (nextLocale: LocaleCode) => {
    if (busyLocale || nextLocale === locale) return;
    setBusyLocale(nextLocale);
    try {
      await setLocale(nextLocale);
    } finally {
      setBusyLocale(null);
    }
  };

  return (
    <div className="w-full rounded-2xl border border-border/70 bg-background/55 p-1.5 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] sm:max-w-[220px]">
      <p className="px-2 pb-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-muted">
        {t("Language")}
      </p>
      <div className="grid grid-cols-2 gap-1">
        {localeOptions.map((code) => {
          const active = locale === code;
          return (
            <button
              key={code}
              type="button"
              onClick={() => void handleLocaleChange(code)}
              disabled={Boolean(busyLocale)}
              className={`inline-flex min-h-10 items-center justify-center gap-1 rounded-xl px-3 text-sm font-medium transition-all ${
                active
                  ? "bg-primary text-white shadow-[0_14px_32px_rgba(124,88,255,0.32)]"
                  : "text-muted hover:bg-surface-hover hover:text-foreground"
              }`}
            >
              <span>{t(`Locale ${code.toUpperCase()}`)}</span>
              {active && <Check size={14} />}
            </button>
          );
        })}
      </div>
    </div>
  );
}

export default function LoginPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const setToken = useAuthStore((s) => s.setToken);

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "auth/login",
    method: "post",
    extraPayload: { client_type: "web" },
    fields: [
      {
        name: "username",
        type: "text",
        label: t("Username"),
        placeholder: t("Enter your username"),
        required: true,
        span: 2,
      },
      {
        name: "password",
        type: "password",
        label: t("Password"),
        placeholder: t("Enter your password"),
        required: true,
        span: 2,
      },
    ],
    onSuccess: (data: unknown) => {
      const result = data as AdminAuthOutput;
      setToken(result.access_token);
      navigate("/", { replace: true });
    },
  });

  return (
    <div className="relative min-h-screen overflow-hidden bg-background px-4 py-6 sm:px-6">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,rgba(124,88,255,0.14),transparent_34%),radial-gradient(circle_at_bottom_left,rgba(56,189,248,0.08),transparent_28%)]" />
      <div className="pointer-events-none absolute -right-16 top-10 h-72 w-72 rounded-full bg-primary/15 blur-3xl" />
      <div className="pointer-events-none absolute -left-12 bottom-6 h-64 w-64 rounded-full bg-sky-500/10 blur-3xl" />

      <div className="relative mx-auto flex min-h-[calc(100vh-3rem)] max-w-5xl items-center justify-center">
        <div className="w-full max-w-[540px] overflow-hidden rounded-[28px] border border-border/70 bg-surface/85 shadow-[0_30px_90px_rgba(2,8,23,0.55)] backdrop-blur">
          <div className="border-b border-border/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.03),rgba(255,255,255,0))] p-6 sm:p-8">
            <div className="flex flex-col gap-6 sm:flex-row sm:items-start sm:justify-between">
              <div className="max-w-xs">
                <div className="mb-4 inline-flex h-12 w-12 items-center justify-center rounded-2xl border border-border/70 bg-background/60 text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
                  <Shield size={20} />
                </div>
                <h1 className="text-3xl font-semibold tracking-tight text-foreground sm:text-[2.1rem]">
                  {t("Admin Portal")}
                </h1>
                <p className="mt-2 text-sm leading-6 text-muted">
                  {t("Sign in to your account")}
                </p>
              </div>

              <LoginLocaleSelector />
            </div>
          </div>

          <div className="p-6 sm:p-8">
            <div className="rounded-2xl border border-border/60 bg-background/35 p-5 sm:p-6">
              {errors.general && (
                <div className="mb-4 rounded-lg bg-error-muted px-3 py-2 text-sm text-error">
                  {errors.general}
                </div>
              )}

              {form}

              <Button
                type="button"
                onClick={() => {
                  void submit();
                }}
                busy={busy}
                variant="primary"
                className="mt-4 h-12 w-full shadow-[0_18px_34px_rgba(124,88,255,0.28)]"
              >
                {busy ? t("Signing in...") : t("Sign in")}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
