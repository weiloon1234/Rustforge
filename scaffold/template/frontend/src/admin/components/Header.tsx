import { useEffect, useMemo, useRef, useState } from "react";
import {
  Menu,
  ChevronDown,
  ChevronUp,
  Check,
  User,
  Shield,
  LogOut,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AdminMeOutput, AdminProfileUpdateOutput } from "@admin/types";
import {
  Button,
  useAutoForm,
  useLocaleStore,
  useModalStore,
  alertError,
  alertSuccess,
} from "@shared/components";
import { useAuthStore } from "@admin/stores/auth";
import { api } from "@admin/api";
import type { LocaleCode } from "@shared/types/platform";
import { adminLocalePersistence } from "@admin/locale";
import { useRealtimeStore } from "@admin/stores/realtime";
import type { RealtimeStatus } from "@shared/createRealtimeStore";

const wsStatusConfig: Record<RealtimeStatus, { color: string; label: string }> = {
  disconnected: { color: "bg-red-500", label: "Offline" },
  connecting: { color: "bg-yellow-500", label: "Connecting" },
  authenticating: { color: "bg-yellow-500", label: "Authenticating" },
  connected: { color: "bg-green-500", label: "Live" },
  reconnecting: { color: "bg-orange-500", label: "Reconnecting" },
};

function ProfileModal({
  account,
  onUpdated,
  formId,
  onBusyChange,
}: {
  account: AdminMeOutput;
  onUpdated: (next: AdminProfileUpdateOutput) => void;
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
        required: true,
      },
      {
        name: "email",
        type: "email",
        label: t("Email"),
        placeholder: t("Enter email"),
        required: false,
      },
    ],
    defaults: {
      name: account.name,
      email: account.email ?? "",
    },
    onSuccess: (data) => {
      onUpdated(data as AdminProfileUpdateOutput);
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
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {errors.general}
        </p>
      )}
      {form}
    </form>
  );
}

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
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {errors.general}
        </p>
      )}
      {form}
    </form>
  );
}

export default function Header({
  collapsed,
  onToggle,
}: {
  collapsed: boolean;
  onToggle: () => void;
}) {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);
  const setAccount = useAuthStore((s) => s.setAccount);
  const logout = useAuthStore((s) => s.logout);
  const locale = useLocaleStore((s) => s.locale);
  const defaultLocale = useLocaleStore((s) => s.defaultLocale);
  const availableLocales = useLocaleStore((s) => s.availableLocales);
  const [menuOpen, setMenuOpen] = useState(false);
  const [localeBusy, setLocaleBusy] = useState<LocaleCode | null>(null);
  const [logoutBusy, setLogoutBusy] = useState(false);
  const dropdownRef = useRef<HTMLDivElement | null>(null);

  const localeOptions = useMemo<LocaleCode[]>(() => {
    if (availableLocales.length > 0) return availableLocales;
    return [defaultLocale];
  }, [availableLocales, defaultLocale]);

  useEffect(() => {
    if (!menuOpen) return;

    const onClickOutside = (event: MouseEvent) => {
      const target = event.target as Node;
      if (!dropdownRef.current?.contains(target)) {
        setMenuOpen(false);
      }
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setMenuOpen(false);
      }
    };

    document.addEventListener("mousedown", onClickOutside);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onClickOutside);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [menuOpen]);

  const openProfileModal = () => {
    if (!account) return;
    setMenuOpen(false);
    const formId = `admin-profile-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
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
              scopes: account.scopes,
            })
          }
        />
      ),
      footer: renderFooter(false),
    });
  };

  const openSecurityModal = () => {
    setMenuOpen(false);
    const formId = `admin-security-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
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
      const result = await adminLocalePersistence.changeAndPersist(nextLocale);
      if (!result.ok) {
        void alertError({
          title: t("Error"),
          message: t("Failed to update locale."),
        });
      }
    } finally {
      setLocaleBusy(null);
      setMenuOpen(false);
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
      window.location.href = "/admin/login";
    }
  };

  const accountName = account?.name ?? t("Admin");
  const wsStatus = useRealtimeStore((s) => s.status);
  const wsInfo = wsStatusConfig[wsStatus];

  return (
    <header className="rf-header">
      <Button
        onClick={onToggle}
        variant="plain"
        size="sm"
        iconOnly
        className="rounded-lg p-2 text-muted transition-colors hover:bg-surface-hover hover:text-foreground"
        aria-label={collapsed ? t("Expand sidebar") : t("Collapse sidebar")}
      >
        <Menu size={20} />
      </Button>

      <div className="flex-1" />

      <div className="flex items-center gap-1.5 px-2 text-xs text-muted" title={t(wsInfo.label)}>
        <span className={`inline-block h-2 w-2 rounded-full ${wsInfo.color}`} />
        <span>{t(wsInfo.label)}</span>
      </div>

      <div ref={dropdownRef} className="relative">
        <Button
          type="button"
          onClick={() => setMenuOpen((open) => !open)}
          variant="plain"
          size="sm"
          className="inline-flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted transition-colors hover:bg-surface-hover hover:text-foreground"
          aria-label={t("Open account menu")}
          aria-expanded={menuOpen}
        >
          <span>{accountName}</span>
          {menuOpen ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        </Button>

        {menuOpen && (
          <div className="absolute right-0 z-40 mt-2 w-72 rounded-xl border border-border bg-surface p-2 shadow-xl">
            <p className="px-2 py-1 text-xs font-medium uppercase tracking-wide text-muted">
              {t("Language")}
            </p>
            <div className="mb-2 space-y-1">
              {localeOptions.map((code) => (
                <Button
                  key={code}
                  type="button"
                  onClick={() => void handleLocaleChange(code)}
                  busy={localeBusy === code}
                  disabled={Boolean(localeBusy) || logoutBusy}
                  variant="plain"
                  size="sm"
                  className="flex w-full items-center justify-between rounded-lg px-2 py-2 text-sm transition-colors hover:bg-surface-hover"
                >
                  <span>{t(`Locale ${code.toUpperCase()}`)}</span>
                  {locale === code && (
                    <Check size={16} className="text-primary" />
                  )}
                </Button>
              ))}
            </div>

            <div className="my-2 border-t border-border" />

            <Button
              type="button"
              onClick={openProfileModal}
              disabled={Boolean(localeBusy) || logoutBusy}
              variant="plain"
              size="sm"
              className="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-sm transition-colors hover:bg-surface-hover"
            >
              <User size={16} />
              {t("Edit Profile")}
            </Button>
            <Button
              type="button"
              onClick={openSecurityModal}
              disabled={Boolean(localeBusy) || logoutBusy}
              variant="plain"
              size="sm"
              className="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-sm transition-colors hover:bg-surface-hover"
            >
              <Shield size={16} />
              {t("Account Security")}
            </Button>
            <Button
              type="button"
              onClick={() => void handleLogout()}
              busy={logoutBusy}
              disabled={Boolean(localeBusy) || logoutBusy}
              variant="plain"
              size="sm"
              className="mt-1 flex w-full items-center gap-2 rounded-lg px-2 py-2 text-sm transition-colors hover:bg-surface-hover"
            >
              <LogOut size={16} />
              {t("Logout")}
            </Button>
          </div>
        )}
      </div>
    </header>
  );
}
