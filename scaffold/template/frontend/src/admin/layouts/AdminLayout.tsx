import { useState, useEffect, useCallback } from "react";
import { Outlet } from "react-router-dom";
import Sidebar from "@admin/components/Sidebar";
import Header from "@admin/components/Header";
import { ModalOutlet } from "@shared/components";
import { useLocaleStore } from "@shared/components";
import { useAuthStore } from "@admin/stores/auth";
import type { LocaleCode } from "@shared/types/platform";

const STORAGE_KEY = "admin-sidebar-collapsed";
const MOBILE_BREAKPOINT = 768;

function useIsMobile() {
  const [mobile, setMobile] = useState(() => window.innerWidth < MOBILE_BREAKPOINT);
  useEffect(() => {
    const mq = window.matchMedia(`(max-width: ${MOBILE_BREAKPOINT - 1}px)`);
    const handler = (e: MediaQueryListEvent) => setMobile(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);
  return mobile;
}

function resolveBrowserLocale(available: LocaleCode[]): LocaleCode | null {
  if (available.length === 0 || typeof navigator === "undefined") {
    return null;
  }

  const raw = navigator.language?.trim().toLowerCase();
  if (!raw) return null;

  const direct = available.find((locale) => locale.toLowerCase() === raw);
  if (direct) return direct;

  const base = raw.split("-")[0];
  if (!base) return null;
  return available.find((locale) => locale.toLowerCase() === base) ?? null;
}

export default function AdminLayout() {
  const isMobile = useIsMobile();
  const [collapsed, setCollapsed] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });
  const [mobileOpen, setMobileOpen] = useState(false);
  const account = useAuthStore((s) => s.account);
  const locale = useLocaleStore((s) => s.locale);
  const setLocale = useLocaleStore((s) => s.setLocale);
  const availableLocales = useLocaleStore((s) => s.availableLocales);
  const defaultLocale = useLocaleStore((s) => s.defaultLocale);

  useEffect(() => {
    if (!isMobile) localStorage.setItem(STORAGE_KEY, String(collapsed));
  }, [collapsed, isMobile]);

  // Close mobile sidebar on route change
  useEffect(() => {
    if (isMobile) setMobileOpen(false);
  }, [isMobile]);

  useEffect(() => {
    const accountLocale = account?.locale ?? null;
    const normalizedAccountLocale = accountLocale
      ? availableLocales.find(
          (localeOption) => localeOption.toLowerCase() === accountLocale.toLowerCase(),
        )
      : null;
    const browserLocale = resolveBrowserLocale(availableLocales);
    const targetLocale = normalizedAccountLocale ?? browserLocale ?? defaultLocale;
    if (targetLocale !== locale) {
      void setLocale(targetLocale);
    }
  }, [account?.locale, availableLocales, defaultLocale, locale, setLocale]);

  const toggleSidebar = useCallback(() => {
    if (isMobile) {
      setMobileOpen((o) => !o);
    } else {
      setCollapsed((c) => !c);
    }
  }, [isMobile]);

  const sidebarVisible = isMobile ? mobileOpen : true;

  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header collapsed={isMobile ? true : collapsed} onToggle={toggleSidebar} />

      {/* Mobile backdrop */}
      {isMobile && mobileOpen && (
        <div
          className="fixed inset-0 z-20 bg-black/50"
          onClick={() => setMobileOpen(false)}
        />
      )}

      {sidebarVisible && <Sidebar collapsed={isMobile ? false : collapsed} />}

      <main
        className="pt-14 transition-all duration-200"
        style={{ marginLeft: isMobile ? 0 : collapsed ? "4rem" : "16rem" }}
      >
        <div className="p-6">
          <Outlet />
        </div>
      </main>
      <ModalOutlet />
    </div>
  );
}
