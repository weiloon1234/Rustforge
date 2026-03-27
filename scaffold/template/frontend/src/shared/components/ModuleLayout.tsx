import { useState, type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, PanelLeftOpen, X } from "lucide-react";
import { ModalOutlet } from "@shared/components";

export interface ModuleLayoutProps {
  /** Page title shown in the header */
  title: string;
  /** Optional subtitle below title */
  subtitle?: string;
  /** Navigate back to this path when back button is clicked */
  backTo: string;
  /** Optional actions rendered on the right side of the header */
  headerActions?: ReactNode;
  /** Optional left sidebar panel content */
  sidebar?: ReactNode;
  /** Width of the sidebar panel (default: 280px) */
  sidebarWidth?: number;
  /** Main area content */
  children: ReactNode;
}

export function ModuleLayout({
  title,
  subtitle,
  backTo,
  headerActions,
  sidebar,
  sidebarWidth = 280,
  children,
}: ModuleLayoutProps) {
  const navigate = useNavigate();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="flex h-screen flex-col bg-[#f9fafb] text-[#101828]">
      {/* Header */}
      <header className="flex shrink-0 items-center gap-2 border-b border-[#eaecf0] bg-white px-3 py-3 shadow-xs sm:gap-3 sm:px-4 md:px-6">
        {sidebar ? (
          <button
            type="button"
            onClick={() => setSidebarOpen(true)}
            className="inline-flex h-8 w-8 items-center justify-center rounded-lg border border-[#d0d5dd] text-[#344054] transition hover:bg-[#f9fafb] md:hidden"
          >
            <PanelLeftOpen size={16} />
          </button>
        ) : null}
        <button
          type="button"
          onClick={() => navigate(backTo)}
          className="inline-flex h-8 w-8 items-center justify-center rounded-lg border border-[#d0d5dd] text-[#344054] transition hover:bg-[#f9fafb]"
        >
          <ArrowLeft size={16} />
        </button>
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-base font-semibold text-[#101828] sm:text-lg">{title}</h1>
          {subtitle ? (
            <p className="hidden truncate text-sm text-[#667085] sm:block">{subtitle}</p>
          ) : null}
        </div>
        {headerActions ? (
          <div className="flex shrink-0 items-center gap-2">{headerActions}</div>
        ) : null}
      </header>

      {/* Body */}
      <div className="flex min-h-0 flex-1">
        {/* Desktop sidebar */}
        {sidebar ? (
          <aside
            className="hidden shrink-0 overflow-y-auto border-r border-[#eaecf0] bg-white md:block"
            style={{ width: sidebarWidth }}
          >
            {sidebar}
          </aside>
        ) : null}

        {/* Mobile sidebar overlay */}
        {sidebar && sidebarOpen ? (
          <div className="fixed inset-0 z-40 flex md:hidden">
            <div
              className="fixed inset-0 bg-black/30"
              onClick={() => setSidebarOpen(false)}
            />
            <aside
              className="relative z-50 h-full shrink-0 overflow-y-auto bg-white shadow-xl"
              style={{ width: Math.min(sidebarWidth, 300) }}
            >
              <div className="flex items-center justify-end border-b border-[#eaecf0] px-3 py-2">
                <button
                  type="button"
                  onClick={() => setSidebarOpen(false)}
                  className="inline-flex h-7 w-7 items-center justify-center rounded-lg text-[#667085] transition hover:bg-[#f9fafb] hover:text-[#344054]"
                >
                  <X size={16} />
                </button>
              </div>
              {sidebar}
            </aside>
          </div>
        ) : null}

        {/* Main content */}
        <main className="min-w-0 flex-1 overflow-y-auto">{children}</main>
      </div>

      <ModalOutlet />
    </div>
  );
}
