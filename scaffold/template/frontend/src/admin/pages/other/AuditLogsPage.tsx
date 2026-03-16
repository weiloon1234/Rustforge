import { useTranslation } from "react-i18next";
import { Eye } from "lucide-react";
import type {
  AuditLogDatatableRow,
  AuditAction,
} from "@admin/types";
import {
  Button,
  DataTable,
  useModalStore,
  formatDateTime,
} from "@shared/components";

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
              <p className="font-mono text-xs">{log.record_key}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Admin")}
              </p>
              <p>{log.admin_username}</p>
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

  return (
    <DataTable<AuditLogDatatableRow>
      url="datatable/audit_log/query"
      title={t("Audit Logs")}
      subtitle={t("View audit log records")}
      enableAutoRefresh
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
          render: (log) => log.record_key,
        },
        {
          key: "admin_username",
          label: t("Admin"),
          render: (log) => log.admin_username,
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (log) => formatDateTime(log.created_at),
        },
      ]}
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
