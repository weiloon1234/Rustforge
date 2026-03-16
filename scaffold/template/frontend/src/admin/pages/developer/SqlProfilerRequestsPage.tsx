import { Copy, Eye } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type {
  SqlProfilerQueryDatatableRow,
  SqlProfilerRequestDatatableRow,
} from "@admin/types";
import { api } from "@admin/api";
import {
  Button,
  DataTable,
  formatDateTime,
  useModalStore,
} from "@shared/components";

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

function formatDuration(us: number): string {
  if (us < 1000) return `${us} µs`;
  if (us < 1_000_000) return `${(us / 1000).toFixed(2)} ms`;
  return `${(us / 1_000_000).toFixed(2)} s`;
}

function durationUsBadgeClass(us: number): string {
  if (us < 1000) return "text-emerald-700";
  if (us < 10000) return "text-amber-700";
  return "text-red-700";
}

function buildBoundQuery(sql: string, binds: string): string {
  if (!binds) return sql;
  const values = binds.split(", ");
  let result = sql;
  for (let i = values.length; i >= 1; i--) {
    const val = values[i - 1];
    // Rust Display already quotes strings ('value'), leaves numbers/bool/NULL bare.
    // UUIDs and timestamps are bare but need quoting for valid SQL.
    const isQuoted = val.startsWith("'") && val.endsWith("'");
    const isBare = /^-?\d+(\.\d+)?$/.test(val) || val === "true" || val === "false" || val === "NULL";
    const replacement = isQuoted || isBare ? val : `'${val}'`;
    result = result.split(`$${i}`).join(replacement);
  }
  return result;
}

function QueriesModalContent({
  requestId,
}: {
  requestId: string;
}) {
  const { t } = useTranslation();
  const [queries, setQueries] = useState<SqlProfilerQueryDatatableRow[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api
      .post("/datatable/sql-profiler-query/query", {
        base: { page: 1, per_page: 100, include_meta: false },
        "f-request_id": requestId,
      })
      .then((res) => setQueries(res.data.data?.records ?? []))
      .catch(() => setQueries([]))
      .finally(() => setLoading(false));
  }, [requestId]);

  if (loading) {
    return <p className="text-sm text-muted">{t("Loading...")}</p>;
  }

  if (queries.length === 0) {
    return <p className="text-sm text-muted">{t("No queries found.")}</p>;
  }

  return (
    <div className="space-y-2">
      {queries.map((q, i) => (
        <div
          key={q.id}
          className="rounded-lg border border-border bg-surface px-3 py-2 text-xs"
        >
          <div className="mb-1 flex items-center gap-2">
            <span className="text-muted">#{i + 1}</span>
            <span
              className={`inline-block rounded-full px-2 py-0.5 font-medium ${operationBadgeClass(q.operation)}`}
            >
              {q.operation}
            </span>
            <span className="font-mono text-muted">{q.table_name}</span>
            <span className={`ml-auto font-medium ${durationUsBadgeClass(Number(q.duration_us))}`}>
              {formatDuration(Number(q.duration_us))}
            </span>
            <button
              type="button"
              className="ml-1 rounded p-1 text-muted hover:bg-surface-hover hover:text-foreground"
              title={t("Copy query with values")}
              onClick={() => {
                navigator.clipboard.writeText(buildBoundQuery(q.sql, q.binds));
              }}
            >
              <Copy size={14} />
            </button>
          </div>
          <pre className="max-h-32 overflow-auto whitespace-pre-wrap break-all text-xs">
            {q.sql}
          </pre>
          {q.binds && (
            <pre className="mt-1 max-h-20 overflow-auto whitespace-pre-wrap break-all text-xs text-muted">
              {q.binds}
            </pre>
          )}
        </div>
      ))}
    </div>
  );
}

export default function SqlProfilerRequestsPage() {
  const { t } = useTranslation();

  const openQueriesModal = useCallback(
    (row: SqlProfilerRequestDatatableRow) => {
      useModalStore.getState().open({
        title: `${t("Queries")} — ${row.request_method} ${row.request_path}`,
        size: "xl",
        content: <QueriesModalContent requestId={row.id} />,
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
    },
    [t],
  );

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
              onClick={() => openQueriesModal(row)}
              variant="plain"
              size="sm"
              iconOnly
              title={t("View Queries")}
            >
              <Eye size={16} />
            </Button>
          ),
        },
        {
          key: "id",
          label: t("Request ID"),
          sortable: false,
          render: (row) => (
            <span
              className="max-w-[8rem] truncate font-mono text-xs"
              title={row.id}
            >
              {row.id.slice(0, 8)}…
            </span>
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
            <span
              className={`font-medium ${durationBadgeClass(row.total_duration_ms)}`}
            >
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
