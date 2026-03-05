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
  Download,
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
import { TextInput } from "@shared/components/TextInput";
import { Select } from "@shared/components/Select";
import { Button } from "@shared/components/Button";
import { Checkbox } from "@shared/components/Checkbox";
import { alertError } from "@shared/helpers";
import { hasPermission } from "@shared/permissions";
import {
  DatePickerInput,
  DateTimePickerInput,
  TimePickerInput,
} from "@shared/components/TemporalInput";
import type {
  ApiResponse,
  DataTableQueryResponse,
  DataTableMetaDto,
  DataTableColumnMetaDto,
  DataTableFilterFieldDto,
  DataTableQueryRequestBase,
} from "@shared/types";
import { PERMISSION } from "@shared/types";

const PER_PAGE_OPTIONS = [30, 50, 100, 300, 1000, 3000];
const AUTO_REFRESH_SECONDS = 60;
const AUTO_REFRESH_STORAGE_KEY = "rf-datatable-auto-refresh-enabled";

interface DataTableApiContextValue {
  api: AxiosInstance;
  scopes: string[];
}

const DataTableApiContext = createContext<DataTableApiContextValue | null>(
  null,
);

export function DataTableApiProvider({
  api,
  scopes = [],
  children,
}: {
  api: AxiosInstance;
  scopes?: string[];
  children: ReactNode;
}) {
  return (
    <DataTableApiContext.Provider value={{ api, scopes }}>
      {children}
    </DataTableApiContext.Provider>
  );
}

export function useDataTableApi(): AxiosInstance {
  const value = useContext(DataTableApiContext);
  if (!value) {
    throw new Error(
      "DataTableApiProvider is missing. Wrap your portal app with <DataTableApiProvider api={...}>.",
    );
  }
  return value.api;
}

export function useDataTableScopes(): string[] {
  const value = useContext(DataTableApiContext);
  if (!value) {
    throw new Error(
      "DataTableApiProvider is missing. Wrap your portal app with <DataTableApiProvider api={...}>.",
    );
  }
  return value.scopes;
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
  enableAutoRefresh?: boolean;
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
  filterRows:
    | (DataTableFilterFieldDto | DataTableFilterFieldDto[])[]
    | undefined,
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

function resolveRefreshSlot(
  slot: RefreshSlot | undefined,
  refresh: () => void,
): ReactNode {
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

function readAutoRefreshEnabled(): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.localStorage.getItem(AUTO_REFRESH_STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

function writeAutoRefreshEnabled(enabled: boolean): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(AUTO_REFRESH_STORAGE_KEY, String(enabled));
  } catch {
    // Ignore storage errors (private mode/quota).
  }
}

function buildDatatablePayload(args: {
  page: number;
  perPage: number;
  sortingColumn: string;
  sortingDirection: "asc" | "desc";
  includeMeta: boolean;
  filters: Record<string, string>;
  extraBody?: Record<string, unknown>;
}): Record<string, unknown> {
  const base: DataTableQueryRequestBase = {
    page: args.page,
    per_page: args.perPage,
    include_meta: args.includeMeta,
  };

  if (args.sortingColumn) {
    base.sorting_column = args.sortingColumn;
    base.sorting = args.sortingDirection;
  }

  const filterParams = Object.fromEntries(
    Object.entries(args.filters).filter(([, value]) => value !== ""),
  );

  return {
    base,
    ...(args.extraBody ?? {}),
    ...filterParams,
  };
}

function deriveExportCsvUrl(queryUrl: string): string | null {
  const trimmed = queryUrl.trim();
  if (!trimmed.endsWith("/query")) return null;
  return `${trimmed.slice(0, -"/query".length)}/export/csv`;
}

function fileNameFromContentDisposition(
  headerValue: string | null | undefined,
): string | null {
  if (!headerValue) return null;
  const utf8Match = headerValue.match(/filename\*=UTF-8''([^;]+)/i);
  if (utf8Match?.[1]) {
    try {
      return decodeURIComponent(utf8Match[1]).replace(/^"(.*)"$/, "$1");
    } catch {
      return utf8Match[1].replace(/^"(.*)"$/, "$1");
    }
  }
  const basicMatch = headerValue.match(/filename="?([^\";]+)"?/i);
  return basicMatch?.[1] ?? null;
}

function triggerBlobDownload(blob: Blob, fileName: string): void {
  if (typeof window === "undefined") return;
  const objectUrl = window.URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = objectUrl;
  anchor.download = fileName;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.URL.revokeObjectURL(objectUrl);
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
  const translatedPlaceholder = field.placeholder ? t(field.placeholder) : "";
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") onEnter();
  };

  switch (field.type) {
    case "select":
      return (
        <Select
          containerClassName="mb-0"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="py-1.5! text-sm!"
          options={[
            { value: "", label: translatedPlaceholder || t("All") },
            ...(field.options ?? []).map((o) => ({
              value: o.value,
              label: t(o.label),
            })),
          ]}
        />
      );
    case "boolean":
      return (
        <Select
          containerClassName="mb-0"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="py-1.5! text-sm!"
          options={[
            { value: "", label: translatedPlaceholder || t("All") },
            { value: "true", label: t("Yes") },
            { value: "false", label: t("No") },
          ]}
        />
      );
    case "datetime":
      return (
        <DateTimePickerInput
          containerClassName="mb-0"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={translatedPlaceholder}
          className="py-1.5! text-sm!"
        />
      );
    case "date":
      return (
        <DatePickerInput
          containerClassName="mb-0"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={translatedPlaceholder}
          className="py-1.5! text-sm!"
        />
      );
    case "time":
      return (
        <TimePickerInput
          containerClassName="mb-0"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={translatedPlaceholder}
          className="py-1.5! text-sm!"
        />
      );
    case "number":
      return (
        <TextInput
          containerClassName="mb-0"
          type="number"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={translatedPlaceholder}
          className="py-1.5! text-sm!"
        />
      );
    default:
      return (
        <TextInput
          containerClassName="mb-0"
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={translatedPlaceholder}
          className="py-1.5! text-sm!"
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
  enableAutoRefresh = true,
  headerActions,
  headerContent,
  renderTableFooter,
  onPreCall,
  onPostCall,
  footer,
}: DataTableProps<T>) {
  const api = useDataTableApi();
  const scopes = useDataTableScopes();
  const { t } = useTranslation();
  const [data, setData] = useState<DataTableQueryResponse<T> | null>(null);
  const [meta, setMeta] = useState<DataTableMetaDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [exporting, setExporting] = useState(false);
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
  const [autoRefreshEnabled, setAutoRefreshEnabled] = useState<boolean>(() =>
    enableAutoRefresh ? readAutoRefreshEnabled() : false,
  );
  const [countdownSeconds, setCountdownSeconds] =
    useState(AUTO_REFRESH_SECONDS);
  const autoRefreshRequestInFlightRef = useRef(false);
  const autoRefreshEnabledRef = useRef(autoRefreshEnabled);
  const enableAutoRefreshRef = useRef(enableAutoRefresh);
  const exportCsvUrl = deriveExportCsvUrl(url);
  const canExport =
    Boolean(exportCsvUrl) && hasPermission(scopes, PERMISSION.EXPORT);

  useEffect(() => {
    onPreCallRef.current = onPreCall;
  }, [onPreCall]);

  useEffect(() => {
    onPostCallRef.current = onPostCall;
  }, [onPostCall]);

  useEffect(() => {
    filterRowsRef.current = meta?.filter_rows;
  }, [meta?.filter_rows]);

  useEffect(() => {
    autoRefreshEnabledRef.current = autoRefreshEnabled;
  }, [autoRefreshEnabled]);

  useEffect(() => {
    enableAutoRefreshRef.current = enableAutoRefresh;
    if (!enableAutoRefresh) {
      setAutoRefreshEnabled(false);
      setCountdownSeconds(AUTO_REFRESH_SECONDS);
      autoRefreshRequestInFlightRef.current = false;
      return;
    }
    setAutoRefreshEnabled(readAutoRefreshEnabled());
    setCountdownSeconds(AUTO_REFRESH_SECONDS);
  }, [enableAutoRefresh]);

  useEffect(() => {
    if (!enableAutoRefresh) return;
    writeAutoRefreshEnabled(autoRefreshEnabled);
  }, [autoRefreshEnabled, enableAutoRefresh]);

  useEffect(() => {
    if (enableAutoRefresh && autoRefreshEnabled) return;
    setCountdownSeconds(AUTO_REFRESH_SECONDS);
    autoRefreshRequestInFlightRef.current = false;
  }, [autoRefreshEnabled, enableAutoRefresh]);

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

  const indexSortColumn = (() => {
    const preferred = (meta?.defaults?.sorting_column ?? "").trim();
    const preferredMeta = preferred
      ? metaColumns.find((m) => m.name === preferred && m.sortable)
      : undefined;
    if (preferredMeta) return preferredMeta.name;
    const idMeta = metaColumns.find((m) => m.name === "id" && m.sortable);
    return idMeta?.name ?? "";
  })();
  const indexSortable = Boolean(indexSortColumn);

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
      const payload = buildDatatablePayload({
        page: p,
        perPage: pp,
        sortingColumn: sc,
        sortingDirection: (sd || "desc") as "asc" | "desc",
        includeMeta,
        filters,
        extraBody,
      });
      const filterSnapshot = buildFilterSnapshot(
        filterRowsRef.current,
        filters,
      );
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
        const res = await api.post<ApiResponse<DataTableQueryResponse<T>>>(
          url,
          payload,
          {
            signal,
          },
        );
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
        if (enableAutoRefreshRef.current && autoRefreshEnabledRef.current) {
          setCountdownSeconds(AUTO_REFRESH_SECONDS);
        }
      } catch (err) {
        if (err instanceof DOMException && err.name === "AbortError") return;
        onPostCallRef.current?.({
          ...callEvent,
          error: err,
        });
        if (enableAutoRefreshRef.current && autoRefreshEnabledRef.current) {
          setCountdownSeconds(AUTO_REFRESH_SECONDS);
        }
      } finally {
        setLoading(false);
        autoRefreshRequestInFlightRef.current = false;
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
    () =>
      fetchData(
        page,
        perPage,
        sortColumn,
        sortDirection,
        appliedFiltersRef.current,
      ),
    [fetchData, page, perPage, sortColumn, sortDirection],
  );

  const handleExport = useCallback(async () => {
    if (!canExport || !exportCsvUrl || exporting) return;

    setExporting(true);
    const payload = buildDatatablePayload({
      page,
      perPage,
      sortingColumn: sortColumn,
      sortingDirection: sortDirection,
      includeMeta: false,
      filters: appliedFiltersRef.current,
      extraBody,
    });

    try {
      const response = await api.post<Blob>(exportCsvUrl, payload, {
        responseType: "blob",
      });
      const disposition = response.headers["content-disposition"] as
        | string
        | undefined;
      const fileName =
        fileNameFromContentDisposition(disposition) ??
        `datatable-export-${Date.now()}.csv`;
      const blob =
        response.data instanceof Blob
          ? response.data
          : new Blob([response.data], { type: "text/csv; charset=utf-8" });
      triggerBlobDownload(blob, fileName);
    } catch {
      alertError({
        title: t("Error"),
        message: t("Failed to export data."),
      });
    } finally {
      setExporting(false);
    }
  }, [
    api,
    canExport,
    exportCsvUrl,
    exporting,
    extraBody,
    page,
    perPage,
    sortColumn,
    sortDirection,
    t,
  ]);

  useEffect(() => {
    if (
      !enableAutoRefresh ||
      !autoRefreshEnabled ||
      loading ||
      countdownSeconds <= 0
    ) {
      return;
    }

    const timer = window.setInterval(() => {
      setCountdownSeconds((prev) => (prev <= 1 ? 0 : prev - 1));
    }, 1000);

    return () => window.clearInterval(timer);
  }, [autoRefreshEnabled, countdownSeconds, enableAutoRefresh, loading]);

  useEffect(() => {
    if (!enableAutoRefresh || !autoRefreshEnabled || loading) return;
    if (countdownSeconds !== 0) return;
    if (autoRefreshRequestInFlightRef.current) return;
    autoRefreshRequestInFlightRef.current = true;
    void refresh();
  }, [
    autoRefreshEnabled,
    countdownSeconds,
    enableAutoRefresh,
    loading,
    refresh,
  ]);

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

  const handleIndexSort = () => {
    if (!indexSortable || !indexSortColumn) return;
    if (indexSortColumn === displaySortCol) {
      setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"));
    } else {
      setSortDirection("desc");
    }
    setSortColumn(indexSortColumn);
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

  const filterRows = meta?.filter_rows;
  const hasFilters = filterRows && filterRows.length > 0;

  const resolvedHeaderActions = resolveRefreshSlot(headerActions, refresh);
  const resolvedHeaderContent = resolveRefreshSlot(headerContent, refresh);
  const showTopHeader =
    Boolean(title?.trim()) ||
    Boolean(subtitle?.trim()) ||
    Boolean(resolvedHeaderActions) ||
    Boolean(enableAutoRefresh) ||
    canExport ||
    showRefresh;

  return (
    <div>
      {(showTopHeader || resolvedHeaderContent) && (
        <div className="mb-6 space-y-3">
          {showTopHeader && (
            <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <div className="min-w-0 flex-1">
                {title && (
                  <h1 className="text-2xl font-bold">
                    {title}
                  </h1>
                )}
                {subtitle && (
                  <p className="mt-1 text-sm text-muted">{subtitle}</p>
                )}
              </div>
              <div className="flex w-full min-w-0 flex-wrap items-center gap-2 sm:w-auto sm:justify-end">
                {resolvedHeaderActions}
                {enableAutoRefresh && (
                  <div className="w-full max-w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm sm:w-auto">
                    <Checkbox
                      containerClassName="mb-0"
                      wrapperClassName="w-full min-w-0 gap-2"
                      labelClassName="min-w-0 truncate sm:whitespace-nowrap"
                      checked={autoRefreshEnabled}
                      onChange={(e) => setAutoRefreshEnabled(e.target.checked)}
                      label={t("Auto refresh in :seconds seconds", {
                        seconds: countdownSeconds,
                      })}
                    />
                  </div>
                )}
                {canExport && exportCsvUrl && (
                  <Button
                    onClick={() => void handleExport()}
                    busy={exporting}
                    variant="secondary"
                    size="sm"
                  >
                    <Download size={16} />
                    {exporting ? t("Exporting...") : t("Export")}
                  </Button>
                )}
                {showRefresh && (
                  <Button
                    onClick={refresh}
                    disabled={loading}
                    variant="secondary"
                    size="sm"
                  >
                    <RefreshCw
                      size={16}
                      className={loading ? "animate-spin" : ""}
                    />
                    {t("Refresh")}
                  </Button>
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
                  style={{
                    gridTemplateColumns: `repeat(${row.length}, minmax(0, 1fr))`,
                  }}
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
            <Button onClick={applyFilters} variant="primary" size="sm">
              <Search size={14} />
              {t("Search")}
            </Button>
            <Button onClick={resetFilters} variant="secondary" size="sm">
              <X size={14} />
              {t("Reset")}
            </Button>
          </div>
        </div>
      )}

      <div className="rf-dt-shell">
        <div className="rf-dt-scroll">
          <table className="rf-dt-table">
            <thead>
              <tr className="rf-dt-head-row">
                {showIndexColumn && (
                  <th
                    className={`rf-dt-th rf-dt-th-index ${indexSortable ? "rf-dt-th-sortable" : ""}`}
                    onClick={handleIndexSort}
                  >
                    <span className="inline-flex items-center gap-1">
                      {t("#")}
                      {indexSortable &&
                        indexSortColumn === displaySortCol &&
                        displaySortDir === "asc" && <ArrowUp size={14} />}
                      {indexSortable &&
                        indexSortColumn === displaySortCol &&
                        displaySortDir === "desc" && <ArrowDown size={14} />}
                      {indexSortable && indexSortColumn !== displaySortCol && (
                        <ArrowUpDown size={14} className="opacity-30" />
                      )}
                    </span>
                  </th>
                )}
                {renderColumns.map((col) => {
                  const sortable = isColumnSortable(col);
                  const translatedLabel = t(col.label);
                  const displayLabel = translatedLabel.trim()
                    ? translatedLabel
                    : col.label;
                  return (
                    <th
                      key={col.key}
                      className={`rf-dt-th ${
                        sortable ? "rf-dt-th-sortable" : ""
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
                  <td colSpan={99} className="rf-dt-empty-cell">
                    {t("Loading…")}
                  </td>
                </tr>
              )}
              {data && data.records.length === 0 && (
                <tr>
                  <td colSpan={99} className="rf-dt-empty-cell">
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
                      className="rf-dt-row"
                    >
                      {showIndexColumn && (
                        <td className="rf-dt-td rf-dt-td-index">
                          {absoluteIndex + 1}
                        </td>
                      )}
                      {renderColumns.map((col) => {
                        const content = col.render
                          ? col.render(record, {
                              index,
                              absoluteIndex,
                              refresh,
                              record,
                            })
                          : formatCellValue(
                              (record as Record<string, unknown>)[col.key],
                            );

                        return (
                          <td
                            key={col.key}
                            className={`rf-dt-td ${col.cellClassName ?? ""}`}
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
              <tfoot className="rf-dt-foot">
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
      </div>

      {data && (
        <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2 text-sm text-muted">
            <Select
              containerClassName="mb-0"
              value={String(perPage)}
              onChange={(e) => handlePerPageChange(Number(e.target.value))}
              className="w-auto! py-1! pr-8! text-xs!"
              options={PER_PAGE_OPTIONS.map((n) => ({
                value: String(n),
                label: String(n),
              }))}
            />
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
              <Button
                variant="secondary"
                size="xs"
                iconOnly
                disabled={page <= 1}
                onClick={() => goTo(1)}
              >
                <ChevronsLeft size={14} />
              </Button>
              <Button
                variant="secondary"
                size="xs"
                iconOnly
                disabled={page <= 1}
                onClick={() => goTo(page - 1)}
              >
                <ChevronLeft size={14} />
              </Button>
              {buildPageNumbers(page, data.total_pages).map((p, i) =>
                p === "…" ? (
                  <span
                    key={`e${i}`}
                    className="px-1 text-sm text-muted select-none"
                  >
                    …
                  </span>
                ) : (
                  <Button
                    key={p}
                    variant={p === page ? "primary" : "secondary"}
                    size="xs"
                    onClick={() => goTo(p)}
                  >
                    {p}
                  </Button>
                ),
              )}
              <Button
                variant="secondary"
                size="xs"
                iconOnly
                disabled={page >= data.total_pages}
                onClick={() => goTo(page + 1)}
              >
                <ChevronRight size={14} />
              </Button>
              <Button
                variant="secondary"
                size="xs"
                iconOnly
                disabled={page >= data.total_pages}
                onClick={() => goTo(data.total_pages)}
              >
                <ChevronsRight size={14} />
              </Button>
              <div className="ml-2 flex items-center gap-1">
                <TextInput
                  containerClassName="mb-0"
                  type="text"
                  inputMode="numeric"
                  value={jumpValue}
                  onChange={(e) =>
                    setJumpValue(e.target.value.replace(/\D/g, ""))
                  }
                  onKeyDown={(e) => e.key === "Enter" && handleJump()}
                  placeholder={t("Go to")}
                  className="w-16! py-1! text-xs! text-center"
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
