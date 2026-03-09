import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronRight, Loader2, Search, Users } from "lucide-react";
import { Button, TextInput, alertError } from "@shared/components";
import type { AdminDownlineNode, AdminDownlinesOutput, ResolvedUser } from "@admin/types";
import type { ApiResponse } from "@shared/types";
import { api } from "@admin/api";

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

interface StackEntry {
  id: string;
  username: string;
  name: string | null;
}

export default function UserHierarchyPage() {
  const { t } = useTranslation();

  const [searchUsername, setSearchUsername] = useState("");
  const [resolving, setResolving] = useState(false);

  const [stack, setStack] = useState<StackEntry[]>([]);
  const [downlines, setDownlines] = useState<AdminDownlineNode[]>([]);
  const [loading, setLoading] = useState(false);

  const currentParent = stack.length > 0 ? stack[stack.length - 1] : null;

  const fetchDownlines = useCallback(async (parentId: string) => {
    setLoading(true);
    try {
      const res = await api.get<ApiResponse<AdminDownlinesOutput>>(
        `users/hierarchy/${parentId}/downlines`,
      );
      setDownlines(res.data.data.downlines);
    } catch {
      setDownlines([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (currentParent) {
      void fetchDownlines(currentParent.id);
    }
  }, [currentParent?.id, fetchDownlines]);

  const handleSearch = async () => {
    const username = searchUsername.trim();
    if (!username) return;

    setResolving(true);
    try {
      const res = await api.get<ApiResponse<ResolvedUser>>(
        "users/hierarchy/resolve",
        { params: { username } },
      );
      const user = res.data.data;
      setStack([{ id: user.id, username: user.username, name: user.name }]);
      setDownlines([]);
    } catch (error) {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("User not found.")),
      });
    } finally {
      setResolving(false);
    }
  };

  const drillInto = (node: AdminDownlineNode) => {
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
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-lg font-semibold text-foreground">
          {t("User Hierarchy")}
        </h1>
        <p className="text-sm text-muted">
          {t("Browse user referral tree")}
        </p>
      </div>

      {/* Search Bar */}
      <div className="flex items-end gap-3">
        <div className="w-full max-w-sm [&_.rf-field]:mb-0">
          <TextInput
            label={t("Username")}
            type="text"
            placeholder={t("Enter username")}
            value={searchUsername}
            onChange={(e) => setSearchUsername(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                void handleSearch();
              }
            }}
          />
        </div>
        <Button
          onClick={() => void handleSearch()}
          variant="primary"
          size="sm"
          busy={resolving}
          disabled={resolving || !searchUsername.trim()}
        >
          <Search size={16} />
          {t("View Hierarchy")}
        </Button>
      </div>

      {/* Results */}
      {currentParent && (
        <div className="space-y-4">
          {/* Breadcrumb */}
          {stack.length > 1 && (
            <div className="flex flex-wrap items-center gap-1 text-sm text-muted">
              {stack.map((entry, i) => (
                <span key={i} className="inline-flex items-center gap-1">
                  {i > 0 && (
                    <ChevronRight size={14} className="text-muted/50" />
                  )}
                  <button
                    type="button"
                    onClick={() => popTo(i)}
                    disabled={i === stack.length - 1}
                    className={`rounded px-1.5 py-0.5 transition-colors ${
                      i === stack.length - 1
                        ? "font-medium text-primary"
                        : "text-muted hover:text-foreground"
                    }`}
                  >
                    {entry.username}
                  </button>
                </span>
              ))}
            </div>
          )}

          {/* User Info Card */}
          <div className="rounded-lg border border-border bg-surface px-4 py-3">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
                <Users size={18} />
              </div>
              <div>
                <p className="text-sm font-semibold text-foreground">
                  {currentParent.username}
                </p>
                {currentParent.name && (
                  <p className="text-xs text-muted">{currentParent.name}</p>
                )}
              </div>
            </div>
          </div>

          {/* Downlines List */}
          <div className="rounded-lg border border-border bg-surface">
            <div className="border-b border-border px-4 py-3">
              <h3 className="text-sm font-semibold text-foreground">
                {t("Direct Downlines")}
              </h3>
            </div>
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
                    className="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-surface-hover disabled:cursor-default disabled:opacity-70"
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
      )}
    </div>
  );
}
