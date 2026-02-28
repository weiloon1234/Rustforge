import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useAutoForm } from "@shared/useAutoForm";
import { useAuthStore } from "@admin/stores/auth";
import { api } from "@admin/api";
import type { AdminAuthOutput } from "@admin/types";

export default function LoginPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const setToken = useAuthStore((s) => s.setToken);
  const fetchAccount = useAuthStore((s) => s.fetchAccount);

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "/api/v1/admin/auth/login",
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
    onSuccess: async (data: unknown) => {
      const result = data as AdminAuthOutput;
      setToken(result.access_token);
      await fetchAccount();
      navigate("/");
    },
  });

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4">
      <div className="w-full max-w-sm">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            {t("Admin Portal")}
          </h1>
          <p className="mt-1 text-sm text-muted">
            {t("Sign in to your account")}
          </p>
        </div>

        <div className="rounded-xl border border-border bg-surface p-6">
          {errors.general && (
            <div className="mb-4 rounded-lg bg-error-muted px-3 py-2 text-sm text-error">
              {errors.general}
            </div>
          )}

          {form}

          <button
            onClick={submit}
            disabled={busy}
            className="mt-2 w-full rounded-lg bg-primary px-4 py-2.5 text-sm font-medium
              text-primary-foreground transition-colors hover:bg-primary-hover
              disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {busy ? t("Signing in...") : t("Sign in")}
          </button>
        </div>
      </div>
    </div>
  );
}
