import { useState, useEffect } from "react";
import { Outlet } from "react-router-dom";
import Header from "@user/components/Header";
import Sidebar from "@user/components/Sidebar";
import BottomNav from "@user/components/BottomNav";
import { ModalOutlet } from "@shared/components";
import { useAuthStore } from "@user/stores/auth";
import { userLocalePersistence } from "@user/locale";

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

export default function UserLayout() {
  const isMobile = useIsMobile();
  const account = useAuthStore((s) => s.account);

  useEffect(() => {
    void userLocalePersistence.syncFromAccount(account);
  }, [account]);

  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header />

      {!isMobile && <Sidebar />}

      <main
        className="pt-14 transition-all duration-200"
        style={{
          marginLeft: isMobile ? 0 : "14rem",
          paddingBottom: isMobile ? "4.5rem" : 0,
        }}
      >
        <div className="p-6">
          <Outlet />
        </div>
      </main>

      {isMobile && <BottomNav />}

      <ModalOutlet />
    </div>
  );
}
