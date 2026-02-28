import { useState, useEffect, useRef, useCallback, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight, ArrowUp, ArrowDown, ArrowUpDown, Search, X } from "lucide-react";
import type { AxiosInstance } from "axios";
import type { ApiResponse, DataTableQueryResponse, DataTableMetaDto, DataTableColumnMetaDto, DataTableFilterFieldDto } from "@shared/types";

const PER_PAGE_OPTIONS = [30, 50, 100, 300, 1000, 3000];

export interface DataTableSortState {
  column: string;
  direction: "asc" | "desc";
  handleSort: (colName: string) => void;
}

export interface DataTableProps<T> {
  url: string;
  api: AxiosInstance;
  extraBody?: Record<string, unknown>;
  perPage?: number;
  hiddenColumns?: string[];
  prependColumns?: ReactNode | ((sort: DataTableSortState) => ReactNode);
  appendColumns?: ReactNode;
  renderPrependCells?: (record: T, index: number, refresh: () => void) => ReactNode;
  renderAppendCells?: (record: T, index: number, refresh: () => void) => ReactNode;
  columnRenderers?: Record<string, (value: unknown, record: T, refresh: () => void) => ReactNode>;
  rowKey: (record: T) => string | number;
  header?: ReactNode | ((refresh: () => void) => ReactNode);
  footer?: ReactNode;
}

function buildPageNumbers(current: number, total: number): (number | "…")[] {
  if (total <= 7) return Array.from({ length: total }, (_, i) => i + 1);
  const pages: (number | "…")[] = [1];
  const left = Math.max(2, current - 1);
  const right = Math.min(total - 1, current + 1);
  if (left > 2) pages.push("…");
  for (let i = left; i <= right; i++) pages.push(i);
  if (right < total - 1) pages.push("…");
  pages.push(total);
  return pages;
}

function formatCellValue(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

function FilterField({
  field,
  value,
  onChange,
  onEnter,
}: {
  field: DataTableFilterFieldDto;
  value: string;
  onChange: (v: string) => void;
  onEnter: () => void;
}) {
  const { t } = useTranslation();
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") onEnter();
  };

  switch (field.type) {
    case "select":
      return (
        <select
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="rf-select !py-1.5 !text-sm"
        >
          <option value="">{field.placeholder ?? t("All")}</option>
          {(field.options ?? []).map((o) => (
            <option key={o.value} value={o.value}>{t(o.label)}</option>
          ))}
        </select>
      );
    case "boolean":
      return (
        <select
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="rf-select !py-1.5 !text-sm"
        >
          <option value="">{field.placeholder ?? t("All")}</option>
          <option value="true">{t("Yes")}</option>
          <option value="false">{t("No")}</option>
        </select>
      );
    case "datetime":
      return (
        <input
          type="datetime-local"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={field.placeholder ?? ""}
          className="rf-input !py-1.5 !text-sm"
        />
      );
    case "date":
      return (
        <input
          type="date"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={field.placeholder ?? ""}
          className="rf-input !py-1.5 !text-sm"
        />
      );
    case "number":
      return (
        <input
          type="number"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={field.placeholder ?? ""}
          className="rf-input !py-1.5 !text-sm"
        />
      );
    default:
      return (
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={field.placeholder ?? ""}
          className="rf-input !py-1.5 !text-sm"
        />
      );
  }
}

export function DataTable<T>({
  url,
  api,
  extraBody,
  perPage: defaultPerPage = 30,
  hiddenColumns = [],
  prependColumns,
  appendColumns,
  renderPrependCells,
  renderAppendCells,
  columnRenderers,
  rowKey,
  header,
  footer,
}: DataTableProps<T>) {
  const { t } = useTranslation();
  const [data, setData] = useState<DataTableQueryResponse<T> | null>(null);
  const [meta, setMeta] = useState<DataTableMetaDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState(defaultPerPage);
  const [jumpValue, setJumpValue] = useState("");
  const metaLoaded = useRef(false);

  const [sortColumn, setSortColumn] = useState("");
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">("desc");
  const [filterValues, setFilterValues] = useState<Record<string, string>>({});
  const appliedFiltersRef = useRef<Record<string, string>>({});
  const [filterVersion, setFilterVersion] = useState(0);

  const visibleColumns: DataTableColumnMetaDto[] = (meta?.columns ?? []).filter(
    (col) => !hiddenColumns.includes(col.name),
  );

  const displaySortCol = sortColumn || meta?.defaults?.sorting_column || "";
  const displaySortDir = sortColumn
    ? sortDirection
    : ((meta?.defaults?.sorted ?? "desc") as "asc" | "desc");

  const fetchData = useCallback(
    async (p: number, pp: number, sc: string, sd: string, filters: Record<string, string>, signal?: AbortSignal) => {
      setLoading(true);
      const includeMeta = !metaLoaded.current;
      const base: Record<string, unknown> = { page: p, per_page: pp, include_meta: includeMeta };
      if (sc) {
        base.sorting_column = sc;
        base.sorting = sd;
      }
      const filterParams = Object.fromEntries(
        Object.entries(filters).filter(([, v]) => v !== ""),
      );
      try {
        const res = await api.post<ApiResponse<DataTableQueryResponse<T>>>(url, {
          base,
          ...extraBody,
          ...filterParams,
        }, { signal });
        setData(res.data.data);
        if (includeMeta && res.data.data.meta) {
          setMeta(res.data.data.meta);
          metaLoaded.current = true;
        }
      } catch (err) {
        if (err instanceof DOMException && err.name === "AbortError") return;
      } finally {
        setLoading(false);
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [api, url],
  );

  useEffect(() => {
    const controller = new AbortController();
    fetchData(page, perPage, sortColumn, sortDirection, appliedFiltersRef.current, controller.signal);
    return () => controller.abort();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, perPage, sortColumn, sortDirection, filterVersion]);

  const refresh = () => fetchData(page, perPage, sortColumn, sortDirection, appliedFiltersRef.current);

  const totalPages = data?.total_pages ?? 1;
  const goTo = (p: number) => setPage(Math.max(1, Math.min(totalPages, p)));

  const handlePerPageChange = (newPerPage: number) => {
    setPerPage(newPerPage);
    setPage(1);
  };

  const handleJump = () => {
    const n = parseInt(jumpValue, 10);
    if (!isNaN(n) && n >= 1 && n <= totalPages) {
      goTo(n);
    }
    setJumpValue("");
  };

  const handleSort = (colName: string) => {
    const col = meta?.columns.find((c) => c.name === colName);
    if (!col?.sortable) return;
    if (colName === displaySortCol) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"));
    } else {
      setSortDirection("desc");
    }
    setSortColumn(colName);
    setPage(1);
  };

  const applyFilters = () => {
    appliedFiltersRef.current = { ...filterValues };
    setFilterVersion((v) => v + 1);
    setPage(1);
  };

  const resetFilters = () => {
    setFilterValues({});
    appliedFiltersRef.current = {};
    setFilterVersion((v) => v + 1);
    setPage(1);
  };

  const updateFilter = (key: string, value: string) => {
    setFilterValues((prev) => ({ ...prev, [key]: value }));
  };

  const pgBtn = "inline-flex items-center justify-center h-8 min-w-8 rounded-lg border border-border bg-surface text-sm font-medium text-foreground transition hover:bg-surface-hover disabled:opacity-40 disabled:pointer-events-none";
  const pgBtnActive = "inline-flex items-center justify-center h-8 min-w-8 rounded-lg bg-primary text-sm font-medium text-primary-foreground";

  const filterRows = meta?.filter_rows;
  const hasFilters = filterRows && filterRows.length > 0;

  return (
    <div>
      {header && (
        <div className="mb-6 flex items-center justify-between">
          <div className="flex-1">{typeof header === "function" ? header(refresh) : header}</div>
          <button
            onClick={refresh}
            disabled={loading}
            className="ml-3 inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2 text-sm font-medium text-foreground transition hover:bg-surface-hover"
          >
            <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
            {t("Refresh")}
          </button>
        </div>
      )}

      {!header && (
        <div className="mb-4 flex justify-end">
          <button
            onClick={refresh}
            disabled={loading}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2 text-sm font-medium text-foreground transition hover:bg-surface-hover"
          >
            <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
            {t("Refresh")}
          </button>
        </div>
      )}

      {hasFilters && (
        <div className="mb-4 space-y-3 rounded-xl border border-border bg-surface p-4">
          {filterRows.map((row, ri) => {
            if (Array.isArray(row)) {
              return (
                <div key={ri} className={`grid gap-3`} style={{ gridTemplateColumns: `repeat(${row.length}, minmax(0, 1fr))` }}>
                  {row.map((field) => (
                    <div key={field.filter_key}>
                      <label className="mb-1 block text-xs font-medium text-muted">{t(field.label)}</label>
                      <FilterField
                        field={field}
                        value={filterValues[field.filter_key] ?? ""}
                        onChange={(v) => updateFilter(field.filter_key, v)}
                        onEnter={applyFilters}
                      />
                    </div>
                  ))}
                </div>
              );
            }
            const field = row as DataTableFilterFieldDto;
            return (
              <div key={field.filter_key}>
                <label className="mb-1 block text-xs font-medium text-muted">{t(field.label)}</label>
                <FilterField
                  field={field}
                  value={filterValues[field.filter_key] ?? ""}
                  onChange={(v) => updateFilter(field.filter_key, v)}
                  onEnter={applyFilters}
                />
              </div>
            );
          })}
          <div className="flex gap-2 pt-1">
            <button
              onClick={applyFilters}
              className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-sm font-medium text-white transition hover:bg-primary/90"
            >
              <Search size={14} />
              {t("Search")}
            </button>
            <button
              onClick={resetFilters}
              className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-1.5 text-sm font-medium text-foreground transition hover:bg-surface-hover"
            >
              <X size={14} />
              {t("Reset")}
            </button>
          </div>
        </div>
      )}

      <div className="overflow-hidden rounded-xl border border-border bg-surface">
        <table className="w-full text-left text-sm">
          <thead>
            <tr className="border-b border-border bg-surface-hover/50">
              {typeof prependColumns === "function"
                ? prependColumns({ column: displaySortCol, direction: displaySortDir, handleSort })
                : prependColumns}
              {visibleColumns.map((col) => (
                <th
                  key={col.name}
                  className={`px-4 py-3 font-medium text-muted ${col.sortable ? "cursor-pointer select-none" : ""}`}
                  onClick={() => handleSort(col.name)}
                >
                  <span className="inline-flex items-center gap-1">
                    {t(col.label)}
                    {col.sortable && col.name === displaySortCol && displaySortDir === "asc" && <ArrowUp size={14} />}
                    {col.sortable && col.name === displaySortCol && displaySortDir === "desc" && <ArrowDown size={14} />}
                    {col.sortable && col.name !== displaySortCol && <ArrowUpDown size={14} className="opacity-30" />}
                  </span>
                </th>
              ))}
              {appendColumns}
            </tr>
          </thead>
          <tbody>
            {loading && !data && (
              <tr>
                <td colSpan={99} className="px-4 py-8 text-center text-muted">
                  {t("Loading…")}
                </td>
              </tr>
            )}
            {data && data.records.length === 0 && (
              <tr>
                <td colSpan={99} className="px-4 py-8 text-center text-muted">
                  {t("No records found.")}
                </td>
              </tr>
            )}
            {data && data.records.length > 0 && data.records.map((record, index) => (
              <tr key={rowKey(record)} className="border-b border-border last:border-0 hover:bg-surface-hover/30">
                {renderPrependCells?.(record, index, refresh)}
                {visibleColumns.map((col) => {
                  const value = (record as Record<string, unknown>)[col.name];
                  if (columnRenderers?.[col.name]) {
                    return columnRenderers[col.name](value, record, refresh);
                  }
                  return (
                    <td key={col.name} className="px-4 py-3 text-foreground">
                      {formatCellValue(value)}
                    </td>
                  );
                })}
                {renderAppendCells?.(record, index, refresh)}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {data && (
        <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2 text-sm text-muted">
            <select
              value={perPage}
              onChange={(e) => handlePerPageChange(Number(e.target.value))}
              className="rf-select !w-auto !py-1 !pr-8 !text-xs"
            >
              {PER_PAGE_OPTIONS.map((n) => (
                <option key={n} value={n}>{n}</option>
              ))}
            </select>
            <span>
              {t("Page :page of :total_pages (:total_records total)", {
                page: data.page,
                total_pages: data.total_pages,
                total_records: data.total_records,
              })}
            </span>
          </div>

          {data.total_pages > 1 && (
            <div className="flex items-center gap-1">
              <button className={pgBtn} disabled={page <= 1} onClick={() => goTo(1)}>
                <ChevronsLeft size={14} />
              </button>
              <button className={pgBtn} disabled={page <= 1} onClick={() => goTo(page - 1)}>
                <ChevronLeft size={14} />
              </button>
              {buildPageNumbers(page, data.total_pages).map((p, i) =>
                p === "…" ? (
                  <span key={`e${i}`} className="px-1 text-sm text-muted select-none">…</span>
                ) : (
                  <button key={p} className={p === page ? pgBtnActive : pgBtn} onClick={() => goTo(p)}>
                    {p}
                  </button>
                ),
              )}
              <button className={pgBtn} disabled={page >= data.total_pages} onClick={() => goTo(page + 1)}>
                <ChevronRight size={14} />
              </button>
              <button className={pgBtn} disabled={page >= data.total_pages} onClick={() => goTo(data.total_pages)}>
                <ChevronsRight size={14} />
              </button>
              <div className="ml-2 flex items-center gap-1">
                <input
                  type="text"
                  inputMode="numeric"
                  value={jumpValue}
                  onChange={(e) => setJumpValue(e.target.value.replace(/\D/g, ""))}
                  onKeyDown={(e) => e.key === "Enter" && handleJump()}
                  placeholder={t("Go to")}
                  className="rf-input !w-16 !py-1 !text-xs text-center"
                />
              </div>
            </div>
          )}
        </div>
      )}

      {footer}
    </div>
  );
}
