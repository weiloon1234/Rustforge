import { Eye } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { HttpClientLogDatatableRow } from "@admin/types";
import { DataTable, formatDateTime, useModalStore } from "@shared/components";
import { api } from "@admin/api";

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
      <p className="text-xs font-semibold uppercase tracking-wide text-muted">{title}</p>
      <pre className="max-h-64 overflow-auto rounded-lg border border-border bg-surface px-3 py-2 text-xs text-foreground">
        {prettyPayload(value)}
      </pre>
    </section>
  );
}

export default function HttpClientLogsPage() {
  const { t } = useTranslation();

  const openDetailModal = (log: HttpClientLogDatatableRow) => {
    useModalStore.getState().open({
      title: t("HTTP Client Log Detail"),
      size: "xl",
      content: (
        <div className="space-y-4 text-sm">
          <div className="grid gap-3 sm:grid-cols-2">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("URL")}
              </p>
              <p className="break-all text-foreground">{log.request_url}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Method")}
              </p>
              <p className="text-foreground">{log.request_method}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Status Code")}
              </p>
              <p className="text-foreground">{log.response_status ?? "—"}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Duration (ms)")}
              </p>
              <p className="text-foreground">{log.duration_ms ?? "—"}</p>
            </div>
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Created At")}
              </p>
              <p className="text-foreground">{formatDateTime(log.created_at)}</p>
            </div>
          </div>

          <JsonPanel title={t("Request Headers")} value={log.request_headers} />
          <JsonPanel title={t("Request Body")} value={log.request_body} />
          <JsonPanel title={t("Response Headers")} value={log.response_headers} />
          <JsonPanel title={t("Response Body")} value={log.response_body} />
        </div>
      ),
    });
  };

  return (
    <DataTable<HttpClientLogDatatableRow>
      url="/api/v1/admin/datatable/http-client-log/query"
      api={api}
      perPage={30}
      hiddenColumns={[
        "id",
        "request_headers",
        "request_body",
        "response_headers",
        "response_body",
      ]}
      prependColumns={<th className="px-4 py-3 font-medium text-muted">{t("Actions")}</th>}
      renderPrependCells={(log) => (
        <td className="px-4 py-3">
          <button
            type="button"
            onClick={() => openDetailModal(log)}
            className="rounded-lg p-1.5 text-muted transition hover:bg-surface-hover hover:text-foreground"
            title={t("View Detail")}
          >
            <Eye size={16} />
          </button>
        </td>
      )}
      columnRenderers={{
        request_url: (value) => (
          <td key="request_url" className="max-w-[36rem] break-all px-4 py-3 text-foreground">
            {String(value)}
          </td>
        ),
        request_method: (value) => {
          const method = String(value ?? "").toUpperCase();
          return (
            <td key="request_method" className="px-4 py-3">
              <span
                className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${methodBadgeClass(method)}`}
              >
                {method || "—"}
              </span>
            </td>
          );
        },
        response_status: (value) => {
          const statusNumber =
            typeof value === "number"
              ? value
              : value === null || value === undefined
                ? null
                : Number(value);
          const display =
            statusNumber === null || Number.isNaN(statusNumber)
              ? "—"
              : String(statusNumber);
          return (
            <td key="response_status" className="px-4 py-3">
              <span
                className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${statusBadgeClass(statusNumber === null || Number.isNaN(statusNumber) ? null : statusNumber)}`}
              >
                {display}
              </span>
            </td>
          );
        },
        duration_ms: (value) => (
          <td key="duration_ms" className="px-4 py-3 tabular-nums text-foreground">
            {value === null || value === undefined ? "—" : `${value} ms`}
          </td>
        ),
        created_at: (value) => (
          <td key="created_at" className="px-4 py-3 tabular-nums text-muted">
            {formatDateTime(value as string)}
          </td>
        ),
      }}
      rowKey={(log) => log.id}
      header={() => (
        <div>
          <h1 className="text-2xl font-bold text-foreground">{t("HTTP Client Logs")}</h1>
          <p className="mt-1 text-sm text-muted">
            {t("Inspect outbound HTTP requests and responses")}
          </p>
        </div>
      )}
    />
  );
}
