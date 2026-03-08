import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ChevronRight,
  Copy,
  Check,
  Share2,
  Users,
  Loader2,
} from "lucide-react";
import { QRCodeSVG } from "qrcode.react";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@user/stores/auth";
import { api } from "@user/api";
import type { DownlineNode, DownlinesOutput } from "@user/types/user-team";
import type { ApiResponse } from "@shared/types";

interface StackEntry {
  id: string | null;
  username: string;
  name: string | null;
}

export default function MyTeamPage() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);

  const referralLink = useMemo(
    () =>
      account?.uuid
        ? `${window.location.origin}/register?ref=${account.uuid}`
        : "",
    [account?.uuid],
  );

  const [copied, setCopied] = useState(false);
  const canShare = typeof navigator.share === "function";

  const handleCopy = async () => {
    if (!referralLink) return;
    try {
      await navigator.clipboard.writeText(referralLink);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // clipboard may not be available
    }
  };

  const handleShare = async () => {
    if (!referralLink) return;
    try {
      await navigator.share({
        title: t("My Team"),
        url: referralLink,
      });
    } catch {
      // user cancelled or share not supported
    }
  };

  /* ── Downline Tree ──────────────────────── */

  const [stack, setStack] = useState<StackEntry[]>(() => [
    {
      id: null,
      username: account?.username ?? "",
      name: account?.name ?? null,
    },
  ]);
  const [downlines, setDownlines] = useState<DownlineNode[]>([]);
  const [loading, setLoading] = useState(true);

  const currentParent = stack[stack.length - 1];

  const fetchDownlines = useCallback(async (parentId: string | null) => {
    setLoading(true);
    try {
      const params: Record<string, string> = {};
      if (parentId) params.parent_user_id = parentId;
      const res = await api.get<ApiResponse<DownlinesOutput>>(
        "team/downlines",
        { params },
      );
      setDownlines(res.data.data.downlines);
    } catch {
      setDownlines([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void fetchDownlines(currentParent.id);
  }, [currentParent.id, fetchDownlines]);

  const drillInto = (node: DownlineNode) => {
    if (node.downline_count <= 0) return;
    setStack((prev) => [
      ...prev,
      { id: node.id, username: node.username, name: node.name },
    ]);
  };

  const popTo = (index: number) => {
    if (index >= stack.length - 1) return;
    setStack((prev) => prev.slice(0, index + 1));
  };

  return (
    <div className="mx-auto max-w-xl space-y-6">
      {/* ── Invite Section ─────────────────── */}
      <div className="rf-me-card flex flex-col items-center py-8">
        <h2 className="mb-4 text-lg font-bold text-foreground">
          {t("Invite Link")}
        </h2>

        {referralLink && (
          <div className="rounded-2xl border border-border/60 bg-background/40 p-4">
            <QRCodeSVG
              value={referralLink}
              size={160}
              bgColor="transparent"
              fgColor="#00f0ff"
              level="M"
            />
          </div>
        )}

        <p className="mt-4 max-w-full break-all px-4 text-center font-mono text-xs text-muted">
          {referralLink || "—"}
        </p>

        <div className="mt-4 flex gap-3">
          <button
            type="button"
            onClick={() => void handleCopy()}
            className="inline-flex items-center gap-1.5 rounded-xl border border-border bg-surface px-4 py-2 text-sm font-medium text-foreground transition-colors hover:bg-surface-hover"
          >
            {copied ? <Check size={16} /> : <Copy size={16} />}
            <span>{copied ? t("Copied!") : t("Copy Link")}</span>
          </button>

          {canShare && (
            <button
              type="button"
              onClick={() => void handleShare()}
              className="inline-flex items-center gap-1.5 rounded-xl border border-border bg-surface px-4 py-2 text-sm font-medium text-foreground transition-colors hover:bg-surface-hover"
            >
              <Share2 size={16} />
              <span>{t("Share")}</span>
            </button>
          )}
        </div>
      </div>

      {/* ── Downlines Section ──────────────── */}
      <div className="rf-me-card">
        <div className="flex items-center gap-3 px-4 py-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Users size={18} />
          </div>
          <span className="text-sm font-bold text-foreground">
            {t("Downlines")}
          </span>
        </div>

        {/* Breadcrumb */}
        {stack.length > 1 && (
          <div className="flex flex-wrap items-center gap-1 px-4 pb-2 text-xs text-muted">
            {stack.map((entry, i) => (
              <span key={i} className="inline-flex items-center gap-1">
                {i > 0 && (
                  <ChevronRight size={12} className="text-muted/50" />
                )}
                <button
                  type="button"
                  onClick={() => popTo(i)}
                  disabled={i === stack.length - 1}
                  className={`rounded px-1 py-0.5 transition-colors ${
                    i === stack.length - 1
                      ? "font-medium text-primary"
                      : "text-muted hover:text-foreground"
                  }`}
                >
                  {i === 0 ? t("Me") : entry.username}
                </button>
              </span>
            ))}
          </div>
        )}

        {/* List */}
        <div className="divide-y divide-border">
          {loading ? (
            <div className="flex items-center justify-center py-10">
              <Loader2 size={20} className="animate-spin text-muted" />
            </div>
          ) : downlines.length === 0 ? (
            <p className="px-4 py-10 text-center text-sm text-muted">
              {t("No downlines yet.")}
            </p>
          ) : (
            downlines.map((node) => (
              <button
                key={node.id}
                type="button"
                onClick={() => drillInto(node)}
                disabled={node.downline_count <= 0}
                className="rf-me-action-row w-full"
              >
                <div className="flex min-w-0 flex-1 flex-col items-start">
                  <span className="text-sm font-medium text-foreground">
                    {node.username}
                  </span>
                  {node.name && (
                    <span className="text-xs text-muted">{node.name}</span>
                  )}
                </div>
                <span className="shrink-0 text-xs text-muted">
                  {t(":count members", { count: node.downline_count })}
                </span>
                {node.downline_count > 0 && (
                  <ChevronRight size={16} className="shrink-0 text-muted" />
                )}
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
