import { useEffect, useMemo, useState } from "react";
import {
  User,
  Users,
  Shield,
  Globe,
  LogOut,
  ChevronRight,
  Check,
  Copy,
} from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Button,
  useAutoForm,
  useLocaleStore,
  useModalStore,
  alertSuccess,
  alertError,
} from "@shared/components";
import { formatPhoneDisplay } from "@shared/countryRuntime";
import { useAuthStore } from "@user/stores/auth";
import { api } from "@user/api";
import { userLocalePersistence } from "@user/locale";
import type { UserMeOutput, UserProfileUpdateOutput } from "@user/types/user-auth";
import type { LocaleCode } from "@shared/types/platform";

/* ── Profile Modal ──────────────────────────────── */

function ProfileModal({
  account,
  onUpdated,
  formId,
  onBusyChange,
}: {
  account: UserMeOutput;
  onUpdated: (next: UserProfileUpdateOutput) => void;
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);

  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "auth/profile_update",
    method: "patch",
    fields: [
      {
        name: "name",
        type: "text",
        label: t("Name"),
        placeholder: t("Enter full name"),
        required: false,
      },
      {
        name: "email",
        type: "email",
        label: t("Email"),
        placeholder: t("Enter email"),
        required: false,
      },
      {
        name: "contact",
        type: "contact",
        span: 2,
      },
    ],
    defaults: {
      name: account.name ?? "",
      email: account.email ?? "",
      country_iso2: account.country_iso2 ?? "",
      contact_number: account.contact_number ?? "",
    },
    onSuccess: (data) => {
      onUpdated(data as UserProfileUpdateOutput);
      close();
      void alertSuccess({
        title: t("Success"),
        message: t("Profile updated successfully"),
      });
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit} className="space-y-4">
      {errors.general && (
        <p className="rounded-lg bg-error-muted px-3 py-2 text-sm text-error">
          {errors.general}
        </p>
      )}
      {form}
    </form>
  );
}

/* ── Security Modal ─────────────────────────────── */

function SecurityModal({
  formId,
  onBusyChange,
}: {
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const close = useModalStore((s) => s.close);
  const { submit, busy, form, errors } = useAutoForm(api, {
    url: "auth/password_update",
    method: "patch",
    fields: [
      {
        name: "current_password",
        type: "password",
        label: t("Current Password"),
        placeholder: t("Enter current password"),
        required: true,
        span: 2,
      },
      {
        name: "password",
        type: "password",
        label: t("New Password"),
        placeholder: t("Enter password"),
        required: true,
      },
      {
        name: "password_confirmation",
        type: "password",
        label: t("Confirm Password"),
        placeholder: t("Confirm new password"),
        required: true,
      },
    ],
    onSuccess: () => {
      close();
      void alertSuccess({
        title: t("Success"),
        message: t("Password updated successfully"),
      });
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit} className="space-y-4">
      {errors.general && (
        <p className="rounded-lg bg-error-muted px-3 py-2 text-sm text-error">
          {errors.general}
        </p>
      )}
      {form}
    </form>
  );
}

/* ── Me Page ────────────────────────────────────── */

export default function MePage() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);
  const setAccount = useAuthStore((s) => s.setAccount);
  const logout = useAuthStore((s) => s.logout);
  const locale = useLocaleStore((s) => s.locale);
  const defaultLocale = useLocaleStore((s) => s.defaultLocale);
  const availableLocales = useLocaleStore((s) => s.availableLocales);
  const [localeBusy, setLocaleBusy] = useState<LocaleCode | null>(null);
  const [logoutBusy, setLogoutBusy] = useState(false);
  const [copied, setCopied] = useState(false);

  const localeOptions = useMemo<LocaleCode[]>(() => {
    if (availableLocales.length > 0) return availableLocales;
    return [defaultLocale];
  }, [availableLocales, defaultLocale]);

  const openProfileModal = () => {
    if (!account) return;
    const formId = `user-profile-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button
          type="button"
          onClick={() => useModalStore.getState().close()}
          variant="secondary"
          disabled={busy}
        >
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Saving…") : t("Save")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Edit Profile"),
      size: "lg",
      content: (
        <ProfileModal
          formId={formId}
          account={account}
          onBusyChange={(busy) => {
            if (!modalId) return;
            useModalStore.getState().update(modalId, { footer: renderFooter(busy) });
          }}
          onUpdated={(next) =>
            setAccount({
              ...account,
              ...next,
            })
          }
        />
      ),
      footer: renderFooter(false),
    });
  };

  const openSecurityModal = () => {
    const formId = `user-security-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button
          type="button"
          onClick={() => useModalStore.getState().close()}
          variant="secondary"
          disabled={busy}
        >
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Saving…") : t("Save")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Account Security"),
      size: "lg",
      content: (
        <SecurityModal
          formId={formId}
          onBusyChange={(busy) => {
            if (!modalId) return;
            useModalStore.getState().update(modalId, { footer: renderFooter(busy) });
          }}
        />
      ),
      footer: renderFooter(false),
    });
  };

  const handleLocaleChange = async (nextLocale: LocaleCode) => {
    if (localeBusy || logoutBusy) return;
    setLocaleBusy(nextLocale);
    try {
      const result = await userLocalePersistence.changeAndPersist(nextLocale);
      if (!result.ok) {
        void alertError({
          title: t("Error"),
          message: t("Failed to update locale."),
        });
      }
    } finally {
      setLocaleBusy(null);
    }
  };

  const handleLogout = async () => {
    if (logoutBusy || localeBusy) return;
    setLogoutBusy(true);
    try {
      await api.post("auth/logout", { client_type: "web" });
    } catch {
      // Always clear local auth state even if revoke call fails.
    } finally {
      logout();
      window.location.href = "/login";
    }
  };

  const handleCopyUuid = async () => {
    if (!account?.uuid) return;
    try {
      await navigator.clipboard.writeText(account.uuid);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard API may not be available
    }
  };

  return (
    <div className="mx-auto max-w-xl space-y-6">
      {/* ── Avatar + Name Card ──────────────── */}
      <div className="rf-me-card flex flex-col items-center py-8">
        <div className="rf-me-avatar">
          <User size={32} />
        </div>
        <h2 className="mt-4 text-xl font-bold text-foreground">
          {account?.name ?? account?.username ?? ""}
        </h2>
        <p className="mt-1 text-sm text-muted">@{account?.username ?? ""}</p>
        {account?.uuid && (
          <button
            type="button"
            onClick={() => void handleCopyUuid()}
            className="mt-2 inline-flex items-center gap-1.5 rounded-lg bg-surface-hover/60 px-3 py-1.5 font-mono text-xs text-primary transition-colors hover:bg-surface-hover"
          >
            <span>{account.uuid}</span>
            {copied ? <Check size={12} /> : <Copy size={12} />}
          </button>
        )}
      </div>

      {/* ── Account Info ────────────────────── */}
      {(account?.email || account?.contact_number) && (
        <div className="rf-me-card divide-y divide-border">
          {account.email && (
            <div className="rf-me-row">
              <span className="text-sm text-muted">{t("Email")}</span>
              <span className="text-sm text-foreground">{account.email}</span>
            </div>
          )}
          {account.contact_number && (
            <div className="rf-me-row">
              <span className="text-sm text-muted">{t("Phone Number")}</span>
              <span className="text-sm text-foreground">
                {formatPhoneDisplay(account.country_iso2, account.contact_number) ?? account.contact_number}
              </span>
            </div>
          )}
        </div>
      )}

      {/* ── Actions ─────────────────────────── */}
      <div className="rf-me-card divide-y divide-border">
        <button
          type="button"
          onClick={openProfileModal}
          className="rf-me-action-row"
        >
          <div className="rf-me-action-icon">
            <User size={18} />
          </div>
          <span className="flex-1 text-left text-sm font-medium text-foreground">
            {t("Edit Profile")}
          </span>
          <ChevronRight size={16} className="text-muted" />
        </button>

        <button
          type="button"
          onClick={openSecurityModal}
          className="rf-me-action-row"
        >
          <div className="rf-me-action-icon">
            <Shield size={18} />
          </div>
          <span className="flex-1 text-left text-sm font-medium text-foreground">
            {t("Account Security")}
          </span>
          <ChevronRight size={16} className="text-muted" />
        </button>

        <Link to="/team" className="rf-me-action-row">
          <div className="rf-me-action-icon">
            <Users size={18} />
          </div>
          <span className="flex-1 text-left text-sm font-medium text-foreground">
            {t("My Team")}
          </span>
          <ChevronRight size={16} className="text-muted" />
        </Link>
      </div>

      {/* ── Language ─────────────────────────── */}
      <div className="rf-me-card">
        <div className="flex items-center gap-3 px-4 py-3">
          <div className="rf-me-action-icon">
            <Globe size={18} />
          </div>
          <span className="flex-1 text-sm font-medium text-foreground">
            {t("Language")}
          </span>
        </div>
        <div className="grid grid-cols-2 gap-2 px-4 pb-4">
          {localeOptions.map((code) => {
            const active = locale === code;
            return (
              <Button
                key={code}
                type="button"
                onClick={() => void handleLocaleChange(code)}
                busy={localeBusy === code}
                disabled={Boolean(localeBusy) || logoutBusy}
                variant={active ? "primary" : "secondary"}
                size="sm"
                className={`justify-center gap-1.5 ${active ? "shadow-[0_8px_20px_rgba(0,240,255,0.2)]" : ""}`}
              >
                <span>{t(`Locale ${code.toUpperCase()}`)}</span>
                {active && <Check size={14} />}
              </Button>
            );
          })}
        </div>
      </div>

      {/* ── Logout ──────────────────────────── */}
      <div className="rf-me-card">
        <button
          type="button"
          onClick={() => void handleLogout()}
          disabled={logoutBusy || Boolean(localeBusy)}
          className="rf-me-action-row text-error"
        >
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-error/10 text-error">
            <LogOut size={18} />
          </div>
          <span className="flex-1 text-left text-sm font-medium">
            {logoutBusy ? t("Logging out...") : t("Logout")}
          </span>
        </button>
      </div>
    </div>
  );
}
