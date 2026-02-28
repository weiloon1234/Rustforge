import { useState, useEffect, useCallback } from "react";
import { Outlet } from "react-router-dom";
import Sidebar from "@admin/components/Sidebar";
import Header from "@admin/components/Header";
import { ModalOutlet } from "@shared/components";

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

export default function AdminLayout() {
  const isMobile = useIsMobile();
  const [collapsed, setCollapsed] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => {
    if (!isMobile) localStorage.setItem(STORAGE_KEY, String(collapsed));
  }, [collapsed, isMobile]);

  // Close mobile sidebar on route change
  useEffect(() => {
    if (isMobile) setMobileOpen(false);
  }, [isMobile]);

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
