import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Eye } from "lucide-react";
import type {
  AuditLogDatatableRow,
  AuditLogDatatableSummaryOutput,
  AdminBatchResolveOutput,
  AuditAction,
} from "@admin/types";
import type { ApiResponse } from "@shared/types";
import {
  Button,
  DataTable,
  useModalStore,
  formatDateTime,
} from "@shared/components";
import type { DataTablePostCallEvent } from "@shared/components";
import { api } from "@admin/api";

const ACTION_COLORS: Record<AuditAction, string> = {
  "1": "bg-emerald-100 text-emerald-700",
  "2": "bg-blue-100 text-blue-700",
  "3": "bg-red-100 text-red-700",
};

function ActionBadge({
  action,
  label,
}: {
  action: AuditAction;
  label: string;
}) {
  return (
    <span
      className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${ACTION_COLORS[action] ?? "bg-gray-100 text-gray-700"}`}
    >
      {label}
    </span>
  );
}

function prettyJson(value: unknown): string {
  if (value === null || value === undefined) return "—";
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function JsonPanel({ title, value }: { title: string; value: unknown }) {
  return (
    <section className="space-y-1">
      <p className="text-xs font-semibold uppercase tracking-wide text-muted">
        {title}
      </p>
      <pre className="max-h-64 overflow-auto rounded-lg border border-border bg-surface px-3 py-2 text-xs">
        {prettyJson(value)}
      </pre>
    </section>
  );
}

function DiffSummary({
  oldData,
  newData,
  t,
}: {
  oldData: Record<string, unknown> | null;
  newData: Record<string, unknown> | null;
  t: (key: string, opts?: Record<string, unknown>) => string;
}) {
  if (!oldData || !newData) return null;
  const changedKeys = Object.keys(newData).filter(
    (key) => JSON.stringify(oldData[key]) !== JSON.stringify(newData[key]),
  );
  if (changedKeys.length === 0) return null;
  return (
    <p className="text-xs text-muted">
      {changedKeys.length} {t("fields changed")}: {changedKeys.join(", ")}
    </p>
  );
}

export default function AuditLogsPage() {
  const { t } = useTranslation();
  const [summary, setSummary] =
    useState<AuditLogDatatableSummaryOutput | null>(null);
  const [adminMap, setAdminMap] = useState<Map<string, string>>(new Map());
  const summaryRequestId = useRef(0);

  const resolveAdminName = (adminId: string): string =>
    adminMap.get(adminId) ?? adminId;

  const openDetailModal = (log: AuditLogDatatableRow) => {
    useModalStore.getState().open({
      title: t("Audit Detail"),
      size: "xl",
      content: (
        <div className="space-y-4 text-sm">
          <div className="grid gap-3 sm:grid-cols-2">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Action")}
              </p>
              <p>
                <ActionBadge action={log.action} label={log.action_explained} />
              </p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Table")}
              </p>
              <p>{log.table_name}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Record ID")}
              </p>
              <p className="font-mono text-xs">{log.record_id}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Admin")}
              </p>
              <p>{resolveAdminName(log.admin_id)}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Created At")}
              </p>
              <p>{formatDateTime(log.created_at)}</p>
            </div>
          </div>

          <DiffSummary oldData={log.old_data} newData={log.new_data} t={t} />
          <JsonPanel title={t("Old Data")} value={log.old_data} />
          <JsonPanel title={t("New Data")} value={log.new_data} />
        </div>
      ),
      footer: (
        <Button
          type="button"
          onClick={() => useModalStore.getState().close()}
          variant="secondary"
        >
          {t("Close")}
        </Button>
      ),
    });
  };

  const handleDatatablePostCall = (
    event: DataTablePostCallEvent<AuditLogDatatableRow>,
  ) => {
    if (!event.response || event.error) {
      setSummary(null);
      return;
    }

    // Fetch summary
    const requestId = ++summaryRequestId.current;
    const payload: Record<string, unknown> = {
      base: { include_meta: false },
      ...event.filters.applied,
    };

    void api
      .post<ApiResponse<AuditLogDatatableSummaryOutput>>(
        "datatable/audit_log/summary",
        payload,
      )
      .then((res) => {
        if (summaryRequestId.current !== requestId) return;
        setSummary(res.data.data);
      })
      .catch(() => {
        if (summaryRequestId.current !== requestId) return;
        setSummary(null);
      });

    // Batch resolve admin IDs
    const rows = event.response?.records ?? [];
    const uniqueIds = [
      ...new Set(rows.map((r: AuditLogDatatableRow) => r.admin_id).filter(Boolean)),
    ];
    const unknownIds = uniqueIds.filter((id: string) => !adminMap.has(id));
    if (unknownIds.length > 0) {
      void api
        .post<ApiResponse<AdminBatchResolveOutput>>("admins/batch_resolve", {
          ids: unknownIds.map(Number),
        })
        .then((res) => {
          setAdminMap((prev) => {
            const next = new Map(prev);
            for (const entry of res.data.data.entries) {
              next.set(entry.id, entry.name || entry.username);
            }
            return next;
          });
        })
        .catch(() => {});
    }
  };

  return (
    <DataTable<AuditLogDatatableRow>
      url="datatable/audit_log/query"
      title={t("Audit Logs")}
      subtitle={t("View audit log records")}
      enableAutoRefresh
      headerContent={
        summary ? (
          <div className="grid gap-2 sm:grid-cols-4">
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">{t("Filtered Total")}</p>
              <p className="font-semibold">{summary.total_filtered}</p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">
                {t("enum.audit_action.create")}
              </p>
              <p className="font-semibold text-emerald-600">
                {summary.create_count}
              </p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">
                {t("enum.audit_action.update")}
              </p>
              <p className="font-semibold text-blue-600">
                {summary.update_count}
              </p>
            </div>
            <div className="rounded-lg border border-border bg-surface px-3 py-2 text-sm">
              <p className="text-xs text-muted">
                {t("enum.audit_action.delete")}
              </p>
              <p className="font-semibold text-red-600">
                {summary.delete_count}
              </p>
            </div>
          </div>
        ) : undefined
      }
      columns={[
        {
          key: "actions",
          label: t("Actions"),
          sortable: false,
          render: (log) => (
            <Button
              type="button"
              onClick={() => openDetailModal(log)}
              variant="plain"
              size="sm"
              iconOnly
              title={t("View")}
            >
              <Eye size={16} />
            </Button>
          ),
        },
        {
          key: "action",
          label: t("Action"),
          render: (log) => (
            <ActionBadge action={log.action} label={log.action_explained} />
          ),
        },
        {
          key: "table_name",
          label: t("Table"),
          render: (log) => log.table_name,
        },
        {
          key: "record_id",
          label: t("Record ID"),
          cellClassName: "font-mono text-xs",
          render: (log) => log.record_id,
        },
        {
          key: "admin_id",
          label: t("Admin"),
          render: (log) => resolveAdminName(log.admin_id),
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (log) => formatDateTime(log.created_at),
        },
      ]}
      onPostCall={handleDatatablePostCall}
      renderTableFooter={({ records }) => (
        <tr>
          <td colSpan={99} className="px-4 py-2 text-xs text-muted">
            {t("Page rows")}: {records.length}
          </td>
        </tr>
      )}
    />
  );
}
