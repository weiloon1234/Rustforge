import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Wallet, Loader2, ArrowDownLeft, ArrowUpRight, Clock, CheckCircle, XCircle, AlertCircle } from "lucide-react";
import { api } from "@user/api";
import { useAuthStore } from "@user/stores/auth";
import { Button } from "@shared/components/Button";
import { TextInput } from "@shared/components/TextInput";
import { Select } from "@shared/components/Select";
import type { ApiResponse, ApiErrorResponse, WithdrawalStatus } from "@shared/types";
import { WITHDRAWAL_STATUS, WITHDRAWAL_STATUS_I18N, CREDIT_TRANSACTION_TYPE_I18N } from "@shared/types";
import type {
  WalletLedgerEntry,
  WalletLedgerResponse,
  CryptoNetworkOption,
  UserWithdrawalOutput,
  UserWithdrawalHistoryResponse,
  WithdrawalConfigResponse,
} from "@user/types";

// ─── Shared helpers ──────────────────────────────────────────────


function parseDate(dateStr: string): Date {
  const iso = dateStr
    .replace(" ", "T")
    .replace(/ \+(\d{2}):(\d{2}):\d{2}$/, "+$1:$2");
  return new Date(iso);
}

function timeAgo(dateStr: string, t: (key: string, opts?: Record<string, unknown>) => string): string {
  const diff = Date.now() - parseDate(dateStr).getTime();
  if (Number.isNaN(diff)) return "";
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return t("Just now");
  if (mins < 60) return t(":count m ago", { count: mins });
  const hours = Math.floor(mins / 60);
  if (hours < 24) return t(":count h ago", { count: hours });
  const days = Math.floor(hours / 24);
  return t(":count d ago", { count: days });
}

type TabKey = "transactions" | "topup" | "withdrawal";

// ─── LedgerCard ──────────────────────────────────────────────────

function LedgerCard({ entry, t }: { entry: WalletLedgerEntry; t: (key: string, opts?: Record<string, unknown>) => string }) {
  const amount = parseFloat(entry.amount);
  const isPositive = amount >= 0;
  const label = CREDIT_TRANSACTION_TYPE_I18N[entry.transaction_type] ?? entry.transaction_type;

  return (
    <div className="flex items-center gap-3 rounded-xl border border-border bg-surface px-4 py-3">
      <span className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-lg ${isPositive ? "bg-green-500/10 text-green-400" : "bg-red-500/10 text-red-400"}`}>
        {isPositive ? <ArrowDownLeft size={16} /> : <ArrowUpRight size={16} />}
      </span>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium truncate">{t(label)}</p>
        <p className="mt-0.5 text-xs text-muted">{timeAgo(entry.created_at, t)}</p>
      </div>
      <span className={`shrink-0 font-semibold ${isPositive ? "text-green-400" : "text-red-400"}`}>
        {isPositive ? "+" : ""}{amount.toFixed(2)}
      </span>
    </div>
  );
}

// ─── TransactionsTab ─────────────────────────────────────────────

function TransactionsTab() {
  const { t } = useTranslation();
  const [items, setItems] = useState<WalletLedgerEntry[]>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const fetchingRef = useRef(false);

  const fetchPage = useCallback(async (cursor: string | null, append: boolean) => {
    if (fetchingRef.current) return;
    fetchingRef.current = true;
    try {
      const params: Record<string, string | number> = { limit: 20 };
      if (cursor) params.cursor = cursor;
      const res = await api.get<ApiResponse<WalletLedgerResponse>>("/wallet/ledger", { params });
      const data = res.data.data;
      setItems((prev) => append ? [...prev, ...data.items] : data.items);
      setNextCursor(data.next_cursor);
      setHasMore(data.next_cursor != null);
    } catch {
      if (!append) setItems([]);
      setHasMore(false);
    } finally {
      setLoading(false);
      setLoadingMore(false);
      fetchingRef.current = false;
    }
  }, []);

  useEffect(() => {
    void fetchPage(null, false);
  }, [fetchPage]);

  useEffect(() => {
    const el = sentinelRef.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !fetchingRef.current) {
          setLoadingMore(true);
          void fetchPage(nextCursor, true);
        }
      },
      { threshold: 0.1 },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, [hasMore, nextCursor, fetchPage]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-16">
        <Loader2 size={32} className="animate-spin text-muted" />
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center rounded-xl border border-border bg-surface py-16">
        <Wallet size={40} className="text-muted" />
        <p className="mt-4 text-sm text-muted">{t("No transactions yet.")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2">
      {items.map((entry) => (
        <LedgerCard key={entry.id} entry={entry} t={t} />
      ))}
      <div ref={sentinelRef} className="h-1" />
      {loadingMore && (
        <div className="flex items-center justify-center py-4">
          <Loader2 size={20} className="animate-spin text-muted" />
        </div>
      )}
    </div>
  );
}

// ─── TopUpTab ────────────────────────────────────────────────────

function TopUpTab() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-border bg-surface py-16">
      <Wallet size={40} className="text-muted" />
      <p className="mt-4 text-sm text-muted">{t("Coming Soon")}</p>
    </div>
  );
}

// ─── WithdrawalTab ───────────────────────────────────────────────

interface StatusStyle { bg: string; text: string; icon: typeof Clock; i18n: string }

function statusStyleFor(status: WithdrawalStatus): StatusStyle {
  switch (status) {
    case WITHDRAWAL_STATUS.PENDING:
      return { bg: "bg-yellow-500/10", text: "text-yellow-400", icon: Clock, i18n: WITHDRAWAL_STATUS_I18N[status] };
    case WITHDRAWAL_STATUS.PROCESSING:
      return { bg: "bg-blue-500/10", text: "text-blue-400", icon: AlertCircle, i18n: WITHDRAWAL_STATUS_I18N[status] };
    case WITHDRAWAL_STATUS.APPROVED:
      return { bg: "bg-green-500/10", text: "text-green-400", icon: CheckCircle, i18n: WITHDRAWAL_STATUS_I18N[status] };
    case WITHDRAWAL_STATUS.REJECTED:
      return { bg: "bg-red-500/10", text: "text-red-400", icon: XCircle, i18n: WITHDRAWAL_STATUS_I18N[status] };
  }
  return { bg: "bg-gray-500/10", text: "text-gray-400", icon: Clock, i18n: WITHDRAWAL_STATUS_I18N[status] ?? status };
}

function getStatusStyle(status: string) {
  return statusStyleFor(status as WithdrawalStatus);
}

function WithdrawalTab() {
  const { t } = useTranslation();
  const account = useAuthStore((s) => s.account);
  const fetchAccount = useAuthStore((s) => s.fetchAccount);

  // Form state
  const [networks, setNetworks] = useState<CryptoNetworkOption[]>([]);
  const [feePercentage, setFeePercentage] = useState(0);
  const [networkId, setNetworkId] = useState("");
  const [walletAddress, setWalletAddress] = useState("");
  const [amount, setAmount] = useState("");
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string[]>>({});

  // History state
  const [historyItems, setHistoryItems] = useState<UserWithdrawalOutput[]>([]);
  const [historyNextCursor, setHistoryNextCursor] = useState<string | null>(null);
  const [historyHasMore, setHistoryHasMore] = useState(true);
  const [historyLoading, setHistoryLoading] = useState(true);
  const [historyLoadingMore, setHistoryLoadingMore] = useState(false);
  const historyFetchingRef = useRef(false);

  // Load crypto networks + fee config on mount
  useEffect(() => {
    api.get<ApiResponse<WithdrawalConfigResponse>>("/wallet/crypto-networks")
      .then((res) => {
        setNetworks(res.data.data.networks);
        setFeePercentage(parseFloat(res.data.data.fee_percentage));
      })
      .catch(() => {});
  }, []);

  // Fetch withdrawal history
  const fetchHistory = useCallback(async (cursor: string | null, append: boolean) => {
    if (historyFetchingRef.current) return;
    historyFetchingRef.current = true;
    try {
      const params: Record<string, string | number> = { limit: 10 };
      if (cursor) params.cursor = cursor;
      const res = await api.get<ApiResponse<UserWithdrawalHistoryResponse>>("/wallet/withdrawal/history", { params });
      const data = res.data.data;
      setHistoryItems((prev) => append ? [...prev, ...data.items] : data.items);
      setHistoryNextCursor(data.next_cursor);
      setHistoryHasMore(data.next_cursor != null);
    } catch {
      if (!append) setHistoryItems([]);
      setHistoryHasMore(false);
    } finally {
      setHistoryLoading(false);
      setHistoryLoadingMore(false);
      historyFetchingRef.current = false;
    }
  }, []);

  useEffect(() => {
    void fetchHistory(null, false);
  }, [fetchHistory]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setFieldErrors({});
    try {
      await api.post<ApiResponse<UserWithdrawalOutput>>("/wallet/withdrawal", {
        crypto_network_id: Number(networkId),
        crypto_wallet_address: walletAddress,
        amount,
        password,
      });
      toast.success(t("Withdrawal request submitted"));
      // Clear form
      setNetworkId("");
      setWalletAddress("");
      setAmount("");
      setPassword("");
      // Refetch history and balance
      setHistoryLoading(true);
      void fetchHistory(null, false);
      void fetchAccount();
    } catch (err) {
      const axiosErr = err as { response?: { status?: number; data?: ApiErrorResponse } };
      if (axiosErr.response?.status === 422 && axiosErr.response.data?.errors) {
        const errs = axiosErr.response.data.errors;
        setFieldErrors(
          Object.fromEntries(
            Object.entries(errs).filter((e): e is [string, string[]] => e[1] != null),
          ),
        );
      } else {
        const msg = axiosErr.response?.data?.message ?? t("Failed to submit withdrawal.");
        toast.error(msg);
      }
    } finally {
      setSubmitting(false);
    }
  };

  const balance = account?.credit_1 ?? "0";
  const parsedAmount = parseFloat(amount) || 0;
  const fee = parsedAmount * feePercentage;
  const netAmount = parsedAmount - fee;

  return (
    <div className="flex flex-col gap-6">
      {/* Withdrawal form */}
      <form onSubmit={handleSubmit} className="rounded-xl border border-border bg-surface p-4 flex flex-col gap-1">
        <Select
          label={t("Network")}
          placeholder={t("Select a network")}
          required
          value={networkId}
          onChange={(e) => setNetworkId(e.target.value)}
          options={networks.map((n) => ({
            value: n.id,
            label: `${n.name} (${n.symbol})`,
          }))}
          errors={fieldErrors.crypto_network_id}
        />
        <TextInput
          label={t("Wallet Address")}
          placeholder={t("Enter wallet address")}
          required
          value={walletAddress}
          onChange={(e) => setWalletAddress(e.target.value)}
          errors={fieldErrors.crypto_wallet_address}
        />
        <TextInput
          label={t("Amount")}
          type="money"
          required
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          notes={t("Available balance: :amount", { amount: parseFloat(balance).toFixed(2) })}
          errors={fieldErrors.amount}
        />
        {parsedAmount > 0 && (
          <div className="rounded-lg border border-border bg-surface-hover p-3 text-sm">
            <div className="flex justify-between">
              <span className="text-muted">{t("Withdrawal Amount")}</span>
              <span>{parsedAmount.toFixed(2)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted">{t("Fee (:percent%)", { percent: (feePercentage * 100).toFixed(0) })}</span>
              <span className="text-red-400">-{fee.toFixed(2)}</span>
            </div>
            <div className="mt-1 border-t border-border pt-1 flex justify-between font-medium">
              <span>{t("You Receive")}</span>
              <span className="text-green-400">{netAmount.toFixed(2)}</span>
            </div>
          </div>
        )}
        <TextInput
          label={t("Password")}
          type="password"
          required
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          errors={fieldErrors.password}
        />
        <div className="mt-2">
          <Button type="submit" variant="primary" busy={submitting}>
            {t("Submit Withdrawal")}
          </Button>
        </div>
      </form>

      {/* Withdrawal history */}
      {historyLoading ? (
        <div className="flex items-center justify-center py-8">
          <Loader2 size={24} className="animate-spin text-muted" />
        </div>
      ) : historyItems.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-border bg-surface py-12">
          <Wallet size={32} className="text-muted" />
          <p className="mt-3 text-sm text-muted">{t("No withdrawals yet.")}</p>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          {historyItems.map((item) => {
            const style = getStatusStyle(item.status);
            const Icon = style.icon;
            return (
              <div key={item.id} className="flex items-center gap-3 rounded-xl border border-border bg-surface px-4 py-3">
                <span className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-lg ${style.bg} ${style.text}`}>
                  <Icon size={16} />
                </span>
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium truncate">{item.crypto_network_name}</p>
                  <p className="mt-0.5 text-xs text-muted">{timeAgo(item.created_at, t)}</p>
                </div>
                <div className="flex flex-col items-end shrink-0">
                  <span className="font-semibold text-red-400">-{parseFloat(item.amount).toFixed(2)}</span>
                  {parseFloat(item.fee) > 0 && (
                    <span className="text-xs text-muted">
                      {t("Fee")}: {parseFloat(item.fee).toFixed(2)} · {t("Received")}: {parseFloat(item.net_amount).toFixed(2)}
                    </span>
                  )}
                  <span className={`text-xs ${style.text}`}>{t(style.i18n)}</span>
                </div>
              </div>
            );
          })}
          {historyHasMore && (
            <div className="flex justify-center pt-2">
              <Button
                variant="secondary"
                size="sm"
                busy={historyLoadingMore}
                onClick={() => {
                  setHistoryLoadingMore(true);
                  void fetchHistory(historyNextCursor, true);
                }}
              >
                {t("Load More")}
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ─── WalletPage (main) ──────────────────────────────────────────

export default function WalletPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<TabKey>("transactions");

  const tabs: { key: TabKey; label: string }[] = [
    { key: "transactions", label: t("Transactions") },
    { key: "topup", label: t("Top Up") },
    { key: "withdrawal", label: t("Withdrawal") },
  ];

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-2xl font-bold">{t("Wallet")}</h1>
        <p className="mt-1 text-sm text-muted">
          {t("View your credit transactions.")}
        </p>
      </div>

      {/* Tab bar */}
      <div className="mb-4 flex gap-2">
        {tabs.map((tab) => (
          <Button
            key={tab.key}
            variant={activeTab === tab.key ? "primary" : "secondary"}
            size="sm"
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </Button>
        ))}
      </div>

      {/* Tab content */}
      {activeTab === "transactions" && <TransactionsTab />}
      {activeTab === "topup" && <TopUpTab />}
      {activeTab === "withdrawal" && <WithdrawalTab />}
    </div>
  );
}
