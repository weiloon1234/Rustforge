import { Eye } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { SqlProfilerQueryDatatableRow } from "@admin/types";
import { Button, DataTable, formatDateTime, useModalStore } from "@shared/components";

function durationBadgeClass(us: number): string {
  if (us < 1000) return "text-emerald-700";
  if (us < 10000) return "text-amber-700";
  return "text-red-700";
}

function formatDuration(us: number): string {
  if (us < 1000) return `${us} µs`;
  if (us < 1_000_000) return `${(us / 1000).toFixed(2)} ms`;
  return `${(us / 1_000_000).toFixed(2)} s`;
}

function operationBadgeClass(op: string): string {
  switch (op.toUpperCase()) {
    case "SELECT":
      return "bg-emerald-100 text-emerald-700";
    case "INSERT":
      return "bg-blue-100 text-blue-700";
    case "UPDATE":
      return "bg-amber-100 text-amber-700";
    case "DELETE":
      return "bg-rose-100 text-rose-700";
    default:
      return "bg-gray-100 text-gray-700";
  }
}

export default function SqlProfilerQueriesPage() {
  const { t } = useTranslation();

  const openDetailModal = (row: SqlProfilerQueryDatatableRow) => {
    useModalStore.getState().open({
      title: t("SQL Query Detail"),
      size: "xl",
      content: (
        <div className="space-y-4 text-sm">
          <div className="grid gap-3 sm:grid-cols-2">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Query ID")}
              </p>
              <p className="font-mono text-xs">{row.id}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Request ID")}
              </p>
              <p className="break-all font-mono text-xs">{row.request_id}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Table")}
              </p>
              <p>{row.table_name}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Operation")}
              </p>
              <p>{row.operation}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Duration")}
              </p>
              <p>{formatDuration(Number(row.duration_us))}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Created At")}
              </p>
              <p>{formatDateTime(row.created_at)}</p>
            </div>
          </div>

          <section className="space-y-1">
            <p className="text-xs font-semibold uppercase tracking-wide text-muted">
              {t("SQL")}
            </p>
            <pre className="max-h-64 overflow-auto rounded-lg border border-border bg-surface px-3 py-2 text-xs">
              {row.sql}
            </pre>
          </section>

          <section className="space-y-1">
            <p className="text-xs font-semibold uppercase tracking-wide text-muted">
              {t("Binds")}
            </p>
            <pre className="max-h-40 overflow-auto rounded-lg border border-border bg-surface px-3 py-2 text-xs">
              {row.binds || "—"}
            </pre>
          </section>
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
    <DataTable<SqlProfilerQueryDatatableRow>
      url="datatable/sql-profiler-query/query"
      title={t("SQL Profiler Queries")}
      subtitle={t("Individual SQL queries captured by the profiler")}
      enableAutoRefresh
      columns={[
        {
          key: "actions",
          label: t("Actions"),
          sortable: false,
          render: (row) => (
            <Button
              type="button"
              onClick={() => openDetailModal(row)}
              variant="plain"
              size="sm"
              iconOnly
              title={t("View Detail")}
            >
              <Eye size={16} />
            </Button>
          ),
        },
        {
          key: "table_name",
          label: t("Table"),
          render: (row) => (
            <span className="font-mono text-xs">{row.table_name}</span>
          ),
        },
        {
          key: "operation",
          label: t("Operation"),
          render: (row) => {
            const op = row.operation.toUpperCase();
            return (
              <span
                className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${operationBadgeClass(op)}`}
              >
                {op}
              </span>
            );
          },
        },
        {
          key: "sql",
          label: t("SQL"),
          render: (row) => (
            <span className="max-w-sm truncate font-mono text-xs" title={row.sql}>
              {row.sql}
            </span>
          ),
        },
        {
          key: "duration_us",
          label: t("Duration"),
          cellClassName: "tabular-nums",
          render: (row) => (
            <span className={`font-medium ${durationBadgeClass(Number(row.duration_us))}`}>
              {formatDuration(Number(row.duration_us))}
            </span>
          ),
        },
        {
          key: "request_id",
          label: t("Request ID"),
          render: (row) => (
            <span className="max-w-[8rem] truncate font-mono text-xs" title={row.request_id}>
              {row.request_id.slice(0, 8)}…
            </span>
          ),
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => formatDateTime(row.created_at),
        },
      ]}
    />
  );
}
