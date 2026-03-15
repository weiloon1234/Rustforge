import { Gamepad2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@user/stores/auth";
import { useRealtimeStore } from "@user/stores/realtime";
import type { RealtimeStatus } from "@shared/createRealtimeStore";

const statusConfig: Record<RealtimeStatus, { color: string; label: string }> = {
  disconnected: { color: "bg-red-500", label: "Disconnected" },
  connecting: { color: "bg-yellow-500", label: "Connecting" },
  authenticating: { color: "bg-yellow-500", label: "Authenticating" },
  connected: { color: "bg-green-500", label: "Connected" },
  reconnecting: { color: "bg-orange-500", label: "Reconnecting" },
};

export default function Header() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);
  const wsStatus = useRealtimeStore((s) => s.status);
  const wsError = useRealtimeStore((s) => s.error);
  const { color, label } = statusConfig[wsStatus];

  return (
    <header className="rf-header">
      <div className="flex items-center gap-2 text-primary">
        <Gamepad2 size={20} />
        <span className="text-sm font-semibold tracking-wide text-foreground">
          {t("User Portal")}
        </span>
      </div>

      <div className="flex-1" />

      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1.5" title={wsError ?? label}>
          <span className={`inline-block h-2 w-2 rounded-full ${color}`} />
          <span className="text-xs text-muted">{label}</span>
        </div>
        <span className="text-sm text-muted">
          {account?.name ?? account?.username ?? ""}
        </span>
      </div>
    </header>
  );
}
