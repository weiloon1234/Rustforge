import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useTranslation } from "react-i18next";
import {
  RefreshCw,
  ChevronsLeft,
  ChevronLeft,
  ChevronRight,
  ChevronsRight,
  ArrowUp,
  ArrowDown,
  ArrowUpDown,
  Search,
  X,
} from "lucide-react";
import type { AxiosInstance } from "axios";
import type {
  ApiResponse,
  DataTableQueryResponse,
  DataTableMetaDto,
  DataTableColumnMetaDto,
  DataTableFilterFieldDto,
} from "@shared/types";

const PER_PAGE_OPTIONS = [30, 50, 100, 300, 1000, 3000];

const DataTableApiContext = createContext<AxiosInstance | null>(null);

export function DataTableApiProvider({
  api,
  children,
}: {
  api: AxiosInstance;
  children: ReactNode;
}) {
  return (
    <DataTableApiContext.Provider value={api}>
      {children}
    </DataTableApiContext.Provider>
  );
}

export function useDataTableApi(): AxiosInstance {
  const api = useContext(DataTableApiContext);
  if (!api) {
    throw new Error(
      "DataTableApiProvider is missing. Wrap your portal app with <DataTableApiProvider api={...}>.",
    );
  }
  return api;
}

export interface DataTableFilterSnapshot {
  all: Record<string, string>;
  applied: Record<string, string>;
}

export interface DataTablePreCallEvent {
  url: string;
  payload: Record<string, unknown>;
  page: number;
  perPage: number;
  sortingColumn: string;
  sortingDirection: "asc" | "desc";
  includeMeta: boolean;
  filters: DataTableFilterSnapshot;
}

export interface DataTablePostCallEvent<T> extends DataTablePreCallEvent {
  response?: DataTableQueryResponse<T>;
  error?: unknown;
}

export interface DataTableFooterContext<T> {
  records: T[];
  visibleColumns: DataTableColumnMetaDto[];
  sumColumn: (column: string, decimals?: number) => number;
  refresh: () => void;
}

export interface DataTableCellContext<T> {
  index: number;
  absoluteIndex: number;
  refresh: () => void;
  record: T;
}

export interface DataTableColumn<T> {
  key: string;
  label: string;
  sortable?: boolean;
  headerClassName?: string;
  cellClassName?: string;
  render?: (record: T, ctx: DataTableCellContext<T>) => ReactNode;
}

type RefreshSlot = ReactNode | ((refresh: () => void) => ReactNode);

export interface DataTableProps<T> {
  url: string;
  extraBody?: Record<string, unknown>;
  perPage?: number;
  columns?: DataTableColumn<T>[];
  rowKey?: (record: T) => string | number | bigint;
  showIndexColumn?: boolean;
  title?: string;
  subtitle?: string;
  showRefresh?: boolean;
  headerActions?: RefreshSlot;
  headerContent?: RefreshSlot;
  renderTableFooter?: (ctx: DataTableFooterContext<T>) => ReactNode;
  onPreCall?: (event: DataTablePreCallEvent) => void;
  onPostCall?: (event: DataTablePostCallEvent<T>) => void;
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

function toColumnLabel(col: DataTableColumnMetaDto): string {
  const explicit = col.label?.trim();
  if (explicit) return explicit;
  return col.name
    .split("_")
    .map((part) => (part ? part[0].toUpperCase() + part.slice(1) : part))
    .join(" ");
}

function flattenFilterKeys(
  filterRows?: (DataTableFilterFieldDto | DataTableFilterFieldDto[])[],
): string[] {
  if (!filterRows) return [];
  const keys = new Set<string>();
  for (const row of filterRows) {
    if (Array.isArray(row)) {
      for (const field of row) {
        keys.add(field.filter_key);
      }
    } else {
      keys.add(row.filter_key);
    }
  }
  return Array.from(keys);
}

function buildFilterSnapshot(
  filterRows: (DataTableFilterFieldDto | DataTableFilterFieldDto[])[] | undefined,
  filters: Record<string, string>,
): DataTableFilterSnapshot {
  const keys = new Set<string>(flattenFilterKeys(filterRows));
  for (const key of Object.keys(filters)) {
    keys.add(key);
  }

  const all: Record<string, string> = {};
  for (const key of Array.from(keys).sort()) {
    all[key] = filters[key] ?? "";
  }

  const applied = Object.fromEntries(
    Object.entries(filters).filter(([, value]) => value !== ""),
  );

  return { all, applied };
}

function parseNumericCell(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string") {
    const parsed = Number(value.replace(/,/g, ""));
    if (Number.isFinite(parsed)) return parsed;
  }
  return null;
}

function resolveRefreshSlot(slot: RefreshSlot | undefined, refresh: () => void): ReactNode {
  if (!slot) return null;
  if (typeof slot === "function") {
    return (slot as (refresh: () => void) => ReactNode)(refresh);
  }
  return slot;
}

function defaultRecordKey(record: unknown): string | number | null {
  if (!record || typeof record !== "object") return null;
  const value = (record as Record<string, unknown>).id;
  if (typeof value === "bigint") return value.toString();
  if (typeof value === "string" || typeof value === "number") return value;
  return null;
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
            <option key={o.value} value={o.value}>
              {t(o.label)}
            </option>
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
  extraBody,
  perPage: defaultPerPage = 30,
  columns,
  rowKey,
  showIndexColumn = true,
  title,
  subtitle,
  showRefresh = true,
  headerActions,
  headerContent,
  renderTableFooter,
  onPreCall,
  onPostCall,
  footer,
}: DataTableProps<T>) {
  const api = useDataTableApi();
  const { t } = useTranslation();
  const [data, setData] = useState<DataTableQueryResponse<T> | null>(null);
  const [meta, setMeta] = useState<DataTableMetaDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState(defaultPerPage);
  const [jumpValue, setJumpValue] = useState("");
  const metaLoaded = useRef(false);
  const rowKeyWarned = useRef(false);
  const onPreCallRef = useRef(onPreCall);
  const onPostCallRef = useRef(onPostCall);
  const filterRowsRef = useRef(meta?.filter_rows);

  const [sortColumn, setSortColumn] = useState("");
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">("desc");
  const [filterValues, setFilterValues] = useState<Record<string, string>>({});
  const appliedFiltersRef = useRef<Record<string, string>>({});
  const [filterVersion, setFilterVersion] = useState(0);

  useEffect(() => {
    onPreCallRef.current = onPreCall;
  }, [onPreCall]);

  useEffect(() => {
    onPostCallRef.current = onPostCall;
  }, [onPostCall]);

  useEffect(() => {
    filterRowsRef.current = meta?.filter_rows;
  }, [meta?.filter_rows]);

  const metaColumns: DataTableColumnMetaDto[] = meta?.columns ?? [];
  const displaySortCol = sortColumn || meta?.defaults?.sorting_column || "";
  const displaySortDir = sortColumn
    ? sortDirection
    : ((meta?.defaults?.sorted ?? "desc") as "asc" | "desc");

  const renderColumns: DataTableColumn<T>[] =
    columns && columns.length > 0
      ? columns
      : metaColumns.map((col) => ({
          key: col.name,
          label: toColumnLabel(col),
          sortable: col.sortable,
        }));

  const isColumnSortable = useCallback(
    (col: DataTableColumn<T>): boolean => {
      const fromMeta = metaColumns.find((m) => m.name === col.key);
      if (!fromMeta?.sortable) return false;
      return col.sortable !== false;
    },
    [metaColumns],
  );

  const fetchData = useCallback(
    async (
      p: number,
      pp: number,
      sc: string,
      sd: string,
      filters: Record<string, string>,
      signal?: AbortSignal,
    ) => {
      setLoading(true);
      const includeMeta = !metaLoaded.current;
      const base: Record<string, unknown> = {
        page: p,
        per_page: pp,
        include_meta: includeMeta,
      };
      if (sc) {
        base.sorting_column = sc;
        base.sorting = sd;
      }
      const filterParams = Object.fromEntries(
        Object.entries(filters).filter(([, v]) => v !== ""),
      );
      const payload: Record<string, unknown> = {
        base,
        ...extraBody,
        ...filterParams,
      };
      const filterSnapshot = buildFilterSnapshot(filterRowsRef.current, filters);
      const callEvent: DataTablePreCallEvent = {
        url,
        payload,
        page: p,
        perPage: pp,
        sortingColumn: sc,
        sortingDirection: (sd || "desc") as "asc" | "desc",
        includeMeta,
        filters: filterSnapshot,
      };
      onPreCallRef.current?.(callEvent);
      try {
        const res = await api.post<ApiResponse<DataTableQueryResponse<T>>>(url, payload, {
          signal,
        });
        setData(res.data.data);
        if (includeMeta && res.data.data.meta) {
          setMeta(res.data.data.meta);
          metaLoaded.current = true;
        }
        const postFilterSnapshot = buildFilterSnapshot(
          res.data.data.meta?.filter_rows ?? filterRowsRef.current,
          filters,
        );
        onPostCallRef.current?.({
          ...callEvent,
          filters: postFilterSnapshot,
          response: res.data.data,
        });
      } catch (err) {
        if (err instanceof DOMException && err.name === "AbortError") return;
        onPostCallRef.current?.({
          ...callEvent,
          error: err,
        });
      } finally {
        setLoading(false);
      }
    },
    [api, extraBody, url],
  );

  useEffect(() => {
    const controller = new AbortController();
    fetchData(
      page,
      perPage,
      sortColumn,
      sortDirection,
      appliedFiltersRef.current,
      controller.signal,
    );
    return () => controller.abort();
  }, [page, perPage, sortColumn, sortDirection, filterVersion, fetchData]);

  const refresh = useCallback(
    () => fetchData(page, perPage, sortColumn, sortDirection, appliedFiltersRef.current),
    [fetchData, page, perPage, sortColumn, sortDirection],
  );

  const sumColumn = useCallback(
    (column: string, decimals = 2) => {
      if (!data) return 0;
      let sum = 0;
      for (const record of data.records) {
        const value = (record as Record<string, unknown>)[column];
        const numeric = parseNumericCell(value);
        if (numeric !== null) {
          sum += numeric;
        }
      }
      const safeDecimals = Number.isFinite(decimals)
        ? Math.max(0, Math.trunc(decimals))
        : 2;
      return Number(sum.toFixed(safeDecimals));
    },
    [data],
  );

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

  const handleSort = (col: DataTableColumn<T>) => {
    if (!isColumnSortable(col)) return;
    if (col.key === displaySortCol) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"));
    } else {
      setSortDirection("desc");
    }
    setSortColumn(col.key);
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

  const resolveRowKey = (record: T, index: number): string | number => {
    if (rowKey) {
      const value = rowKey(record);
      return typeof value === "bigint" ? value.toString() : value;
    }
    const value = defaultRecordKey(record);
    if (value !== null) return value;
    if (!rowKeyWarned.current) {
      rowKeyWarned.current = true;
      console.error(
        "DataTable: rowKey is missing and record.id is unavailable. Provide `rowKey` prop explicitly.",
      );
    }
    return `rf-row-${page}-${index}`;
  };

  const pgBtn =
    "inline-flex items-center justify-center h-8 min-w-8 rounded-lg border border-border bg-surface text-sm font-medium text-foreground transition hover:bg-surface-hover disabled:opacity-40 disabled:pointer-events-none";
  const pgBtnActive =
    "inline-flex items-center justify-center h-8 min-w-8 rounded-lg bg-primary text-sm font-medium text-primary-foreground";

  const filterRows = meta?.filter_rows;
  const hasFilters = filterRows && filterRows.length > 0;

  const resolvedHeaderActions = resolveRefreshSlot(headerActions, refresh);
  const resolvedHeaderContent = resolveRefreshSlot(headerContent, refresh);
  const showTopHeader =
    Boolean(title?.trim()) ||
    Boolean(subtitle?.trim()) ||
    Boolean(resolvedHeaderActions) ||
    showRefresh;

  return (
    <div>
      {(showTopHeader || resolvedHeaderContent) && (
        <div className="mb-6 space-y-3">
          {showTopHeader && (
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="min-w-0 flex-1">
                {title && <h1 className="text-2xl font-bold text-foreground">{title}</h1>}
                {subtitle && <p className="mt-1 text-sm text-muted">{subtitle}</p>}
              </div>
              <div className="flex shrink-0 flex-wrap items-center justify-end gap-2">
                {resolvedHeaderActions}
                {showRefresh && (
                  <button
                    onClick={refresh}
                    disabled={loading}
                    className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2 text-sm font-medium text-foreground transition hover:bg-surface-hover"
                  >
                    <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
                    {t("Refresh")}
                  </button>
                )}
              </div>
            </div>
          )}
          {resolvedHeaderContent && <div>{resolvedHeaderContent}</div>}
        </div>
      )}

      {hasFilters && (
        <div className="mb-4 space-y-3 rounded-xl border border-border bg-surface p-4">
          {filterRows.map((row, ri) => {
            if (Array.isArray(row)) {
              return (
                <div
                  key={ri}
                  className="grid gap-3"
                  style={{ gridTemplateColumns: `repeat(${row.length}, minmax(0, 1fr))` }}
                >
                  {row.map((field) => (
                    <div key={field.filter_key}>
                      <label className="mb-1 block text-xs font-medium text-muted">
                        {t(field.label)}
                      </label>
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
                <label className="mb-1 block text-xs font-medium text-muted">
                  {t(field.label)}
                </label>
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
              {showIndexColumn && (
                <th className="w-12 px-4 py-3 font-medium text-muted">{t("#")}</th>
              )}
              {renderColumns.map((col) => {
                const sortable = isColumnSortable(col);
                const translatedLabel = t(col.label);
                const displayLabel = translatedLabel.trim() ? translatedLabel : col.label;
                return (
                  <th
                    key={col.key}
                    className={`px-4 py-3 font-medium text-muted ${
                      sortable ? "cursor-pointer select-none" : ""
                    } ${col.headerClassName ?? ""}`}
                    onClick={() => handleSort(col)}
                  >
                    <span className="inline-flex items-center gap-1">
                      {displayLabel}
                      {sortable &&
                        col.key === displaySortCol &&
                        displaySortDir === "asc" && <ArrowUp size={14} />}
                      {sortable &&
                        col.key === displaySortCol &&
                        displaySortDir === "desc" && <ArrowDown size={14} />}
                      {sortable && col.key !== displaySortCol && (
                        <ArrowUpDown size={14} className="opacity-30" />
                      )}
                    </span>
                  </th>
                );
              })}
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
            {data &&
              data.records.length > 0 &&
              data.records.map((record, index) => {
                const absoluteIndex = (data.page - 1) * data.per_page + index;
                return (
                  <tr
                    key={resolveRowKey(record, index)}
                    className="border-b border-border last:border-0 hover:bg-surface-hover/30"
                  >
                    {showIndexColumn && (
                      <td className="px-4 py-3 tabular-nums text-muted">{absoluteIndex + 1}</td>
                    )}
                    {renderColumns.map((col) => {
                      const content = col.render
                        ? col.render(record, {
                            index,
                            absoluteIndex,
                            refresh,
                            record,
                          })
                        : formatCellValue((record as Record<string, unknown>)[col.key]);

                      return (
                        <td
                          key={col.key}
                          className={`px-4 py-3 text-foreground ${col.cellClassName ?? ""}`}
                        >
                          {content}
                        </td>
                      );
                    })}
                  </tr>
                );
              })}
          </tbody>
          {data && renderTableFooter && (
            <tfoot className="border-t border-border bg-surface-hover/20">
              {renderTableFooter({
                records: data.records,
                visibleColumns: metaColumns,
                sumColumn,
                refresh,
              })}
            </tfoot>
          )}
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
                <option key={n} value={n}>
                  {n}
                </option>
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
                  <span key={`e${i}`} className="px-1 text-sm text-muted select-none">
                    …
                  </span>
                ) : (
                  <button
                    key={p}
                    className={p === page ? pgBtnActive : pgBtn}
                    onClick={() => goTo(p)}
                  >
                    {p}
                  </button>
                ),
              )}
              <button
                className={pgBtn}
                disabled={page >= data.total_pages}
                onClick={() => goTo(page + 1)}
              >
                <ChevronRight size={14} />
              </button>
              <button
                className={pgBtn}
                disabled={page >= data.total_pages}
                onClick={() => goTo(data.total_pages)}
              >
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
