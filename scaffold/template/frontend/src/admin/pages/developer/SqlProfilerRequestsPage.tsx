import { Eye } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import type { SqlProfilerRequestDatatableRow } from "@admin/types";
import { Button, DataTable, formatDateTime, useModalStore } from "@shared/components";

function methodBadgeClass(method: string): string {
  switch (method.toUpperCase()) {
    case "GET":
      return "bg-emerald-100 text-emerald-700";
    case "POST":
      return "bg-blue-100 text-blue-700";
    case "PUT":
      return "bg-amber-100 text-amber-700";
    case "PATCH":
      return "bg-purple-100 text-purple-700";
    case "DELETE":
      return "bg-rose-100 text-rose-700";
    default:
      return "bg-gray-100 text-gray-700";
  }
}

function durationBadgeClass(ms: number): string {
  if (ms < 50) return "text-emerald-700";
  if (ms < 200) return "text-amber-700";
  return "text-red-700";
}

export default function SqlProfilerRequestsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const openDetailModal = (row: SqlProfilerRequestDatatableRow) => {
    useModalStore.getState().open({
      title: t("SQL Profiler Request Detail"),
      size: "lg",
      content: (
        <div className="space-y-4 text-sm">
          <div className="grid gap-3 sm:grid-cols-2">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Request ID")}
              </p>
              <p className="break-all font-mono text-xs">{row.id}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Method")}
              </p>
              <p>{row.request_method}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Path")}
              </p>
              <p className="break-all">{row.request_path}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Total Queries")}
              </p>
              <p>{row.total_queries}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Total Duration")}
              </p>
              <p>{row.total_duration_ms.toFixed(2)} ms</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Created At")}
              </p>
              <p>{formatDateTime(row.created_at)}</p>
            </div>
          </div>
        </div>
      ),
      footer: (
        <div className="flex gap-2">
          <Button
            type="button"
            onClick={() => {
              useModalStore.getState().close();
              navigate(`/developer/sql-profiler-queries?f-request_id=${row.id}`);
            }}
            variant="primary"
            size="sm"
          >
            {t("View Queries")}
          </Button>
          <Button
            type="button"
            onClick={() => useModalStore.getState().close()}
            variant="secondary"
          >
            {t("Close")}
          </Button>
        </div>
      ),
    });
  };

  return (
    <DataTable<SqlProfilerRequestDatatableRow>
      url="datatable/sql-profiler-request/query"
      title={t("SQL Profiler Requests")}
      subtitle={t("Inspect per-request SQL query performance")}
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
          key: "request_method",
          label: t("Method"),
          render: (row) => {
            const method = String(row.request_method ?? "").toUpperCase();
            return (
              <span
                className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${methodBadgeClass(method)}`}
              >
                {method || "—"}
              </span>
            );
          },
        },
        {
          key: "request_path",
          label: t("Path"),
          render: (row) => (
            <span className="max-w-xs truncate" title={row.request_path}>
              {row.request_path}
            </span>
          ),
        },
        {
          key: "total_queries",
          label: t("Queries"),
          cellClassName: "tabular-nums",
          render: (row) => row.total_queries,
        },
        {
          key: "total_duration_ms",
          label: t("Duration (ms)"),
          cellClassName: "tabular-nums",
          render: (row) => (
            <span className={`font-medium ${durationBadgeClass(row.total_duration_ms)}`}>
              {row.total_duration_ms.toFixed(2)}
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
