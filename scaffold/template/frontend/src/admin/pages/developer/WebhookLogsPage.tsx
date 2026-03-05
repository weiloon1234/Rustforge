import { Eye } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { WebhookLogDatatableRow } from "@admin/types";
import { Button, DataTable, formatDateTime, useModalStore } from "@shared/components";

function statusBadgeClass(status: number | null): string {
  if (status === null) return "bg-gray-100 text-gray-700";
  if (status >= 200 && status < 300) return "bg-emerald-100 text-emerald-700";
  if (status >= 300 && status < 400) return "bg-blue-100 text-blue-700";
  if (status >= 400 && status < 500) return "bg-amber-100 text-amber-700";
  return "bg-red-100 text-red-700";
}

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

function prettyPayload(value: unknown): string {
  if (value === null || value === undefined) return "—";

  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed) return "—";
    try {
      return JSON.stringify(JSON.parse(trimmed), null, 2);
    } catch {
      return value;
    }
  }

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
        {prettyPayload(value)}
      </pre>
    </section>
  );
}

export default function WebhookLogsPage() {
  const { t } = useTranslation();

  const openDetailModal = (log: WebhookLogDatatableRow) => {
    useModalStore.getState().open({
      title: t("Webhook Log Detail"),
      size: "xl",
      content: (
        <div className="space-y-4 text-sm">
          <div className="grid gap-3 sm:grid-cols-2">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Path / URL")}
              </p>
              <p className="break-all">{log.request_url}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Method")}
              </p>
              <p>{log.request_method}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Status Code")}
              </p>
              <p>{log.response_status ?? "—"}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Duration (ms)")}
              </p>
              <p>{log.duration_ms ?? "—"}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Created At")}
              </p>
              <p>{formatDateTime(log.created_at)}</p>
            </div>
          </div>

          <JsonPanel title={t("Request Headers")} value={log.request_headers} />
          <JsonPanel title={t("Request Body")} value={log.request_body} />
          <JsonPanel title={t("Response Body")} value={log.response_body} />
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
    <DataTable<WebhookLogDatatableRow>
      url="datatable/webhook-log/query"
      title={t("Webhook Logs")}
      subtitle={t("Inspect inbound webhook requests and responses")}
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
              title={t("View Detail")}
            >
              <Eye size={16} />
            </Button>
          ),
        },
        {
          key: "request_url",
          label: t("Path / URL"),
          render: (log) => log.request_url,
        },
        {
          key: "request_method",
          label: t("Method"),
          render: (log) => {
            const method = String(log.request_method ?? "").toUpperCase();
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
          key: "response_status",
          label: t("Status Code"),
          render: (log) => {
            const statusNumber =
              typeof log.response_status === "number" ? log.response_status : null;
            const display =
              statusNumber === null || Number.isNaN(statusNumber)
                ? "—"
                : String(statusNumber);
            return (
              <span
                className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${statusBadgeClass(statusNumber === null || Number.isNaN(statusNumber) ? null : statusNumber)}`}
              >
                {display}
              </span>
            );
          },
        },
        {
          key: "duration_ms",
          label: t("Duration (ms)"),
          cellClassName: "tabular-nums",
          render: (log) =>
            log.duration_ms === null || log.duration_ms === undefined
              ? "—"
              : `${log.duration_ms} ms`,
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (log) => formatDateTime(log.created_at),
        },
      ]}
    />
  );
}
