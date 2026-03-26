import {
  useState,
  useEffect,
  useCallback,
  useRef,
  useImperativeHandle,
  forwardRef,
  type ReactNode,
  type Ref,
} from "react";
import { Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useDataTableApi } from "@shared/components/DataTable";
import { Button } from "@shared/components/Button";
import type {
  ApiResponse,
  DataTableQueryResponse,
  DataTableQueryRequestBase,
} from "@shared/types";

export interface InfiniteListProps<T> {
  url: string;
  extraBody?: Record<string, unknown>;
  perPage?: number;
  renderItem: (item: T, index: number) => ReactNode;
  renderEmpty?: () => ReactNode;
  renderLoading?: () => ReactNode;
  renderLoadingMore?: () => ReactNode;
  loadTrigger?: "intersection" | "button";
  layout?: "list" | "grid";
  gridClassName?: string;
  className?: string;
  paginationMode?: "offset" | "cursor";
  sortingColumn?: string;
  sortingDirection?: "asc" | "desc";
  initialFilters?: Record<string, string>;
}

export interface InfiniteListHandle {
  refresh: () => void;
}

type Status = "idle" | "loading" | "loadingMore" | "error" | "done";

function InfiniteListInner<T>(
  {
    url,
    extraBody,
    perPage = 20,
    renderItem,
    renderEmpty,
    renderLoading,
    renderLoadingMore,
    loadTrigger = "intersection",
    layout = "list",
    gridClassName,
    className,
    paginationMode = "offset",
    sortingColumn,
    sortingDirection = "desc",
    initialFilters,
  }: InfiniteListProps<T>,
  ref: Ref<InfiniteListHandle>,
) {
  const api = useDataTableApi();
  const { t } = useTranslation();

  const [records, setRecords] = useState<T[]>([]);
  const [status, setStatus] = useState<Status>("idle");
  const [hasMore, setHasMore] = useState(true);

  const pageRef = useRef(1);
  const cursorRef = useRef<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);

  const filters = initialFilters ?? {};

  const fetchPage = useCallback(
    async (reset: boolean) => {
      abortRef.current?.abort();
      const controller = new AbortController();
      abortRef.current = controller;

      if (reset) {
        pageRef.current = 1;
        cursorRef.current = null;
        setRecords([]);
        setHasMore(true);
        setStatus("loading");
      } else {
        setStatus("loadingMore");
      }

      const base: DataTableQueryRequestBase = {
        page: paginationMode === "offset" ? pageRef.current : undefined,
        per_page: perPage,
        include_meta: false,
        pagination_mode: paginationMode,
      };

      if (paginationMode === "cursor" && cursorRef.current) {
        base.cursor = cursorRef.current;
      }

      if (sortingColumn) {
        base.sorting_column = sortingColumn;
        base.sorting = sortingDirection;
      }

      const filterParams = Object.fromEntries(
        Object.entries(filters).filter(([, v]) => v !== ""),
      );

      const payload = {
        base,
        ...(extraBody ?? {}),
        ...filterParams,
      };

      try {
        const res = await api.post<ApiResponse<DataTableQueryResponse<T>>>(url, payload, {
          signal: controller.signal,
        });
        const data = res.data.data;

        setRecords((prev) => (reset ? data.records : [...prev, ...data.records]));

        if (paginationMode === "cursor") {
          cursorRef.current = data.next_cursor ?? null;
          setHasMore(data.has_more ?? false);
        } else {
          pageRef.current += 1;
          setHasMore(pageRef.current <= data.total_pages);
        }

        setStatus(
          (paginationMode === "cursor"
            ? !(data.has_more ?? false)
            : pageRef.current > data.total_pages)
            ? "done"
            : "idle",
        );
      } catch (err: unknown) {
        if ((err as { name?: string })?.name === "CanceledError") return;
        setStatus("error");
      }
    },
    [api, url, perPage, paginationMode, sortingColumn, sortingDirection, extraBody, filters],
  );

  const refresh = useCallback(() => {
    fetchPage(true);
  }, [fetchPage]);

  const loadMore = useCallback(() => {
    if (status === "idle" && hasMore) {
      fetchPage(false);
    }
  }, [status, hasMore, fetchPage]);

  useImperativeHandle(ref, () => ({ refresh }), [refresh]);

  // Initial load
  useEffect(() => {
    fetchPage(true);
    return () => abortRef.current?.abort();
  }, [fetchPage]);

  // Intersection observer for auto-load
  useEffect(() => {
    if (loadTrigger !== "intersection" || !sentinelRef.current) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          loadMore();
        }
      },
      { rootMargin: "200px" },
    );

    observer.observe(sentinelRef.current);
    return () => observer.disconnect();
  }, [loadTrigger, loadMore]);

  const isEmpty = status !== "loading" && records.length === 0 && !hasMore;

  const containerClass =
    layout === "grid"
      ? gridClassName ?? "grid grid-cols-2 gap-4"
      : "flex flex-col";

  return (
    <div className={className}>
      {/* Initial loading */}
      {status === "loading" && records.length === 0 && (
        renderLoading?.() ?? (
          <div className="flex items-center justify-center py-12 text-muted">
            <Loader2 className="h-6 w-6 animate-spin" />
          </div>
        )
      )}

      {/* Empty state */}
      {isEmpty && (
        renderEmpty?.() ?? (
          <div className="py-12 text-center text-sm text-muted">
            {t("No records found")}
          </div>
        )
      )}

      {/* Records */}
      {records.length > 0 && (
        <div className={containerClass}>
          {records.map((item, idx) => renderItem(item, idx))}
        </div>
      )}

      {/* Loading more */}
      {status === "loadingMore" && (
        renderLoadingMore?.() ?? (
          <div className="flex items-center justify-center py-4 text-muted">
            <Loader2 className="h-5 w-5 animate-spin" />
          </div>
        )
      )}

      {/* Load more button */}
      {loadTrigger === "button" && hasMore && status === "idle" && (
        <div className="flex justify-center py-4">
          <Button variant="secondary" onClick={loadMore}>
            {t("Load More")}
          </Button>
        </div>
      )}

      {/* Error state */}
      {status === "error" && (
        <div className="flex flex-col items-center gap-2 py-4 text-sm text-muted">
          <span>{t("Failed to load data")}</span>
          <Button variant="secondary" size="sm" onClick={refresh}>
            {t("Retry")}
          </Button>
        </div>
      )}

      {/* Intersection sentinel */}
      {loadTrigger === "intersection" && hasMore && <div ref={sentinelRef} />}
    </div>
  );
}

export const InfiniteList = forwardRef(InfiniteListInner) as <T>(
  props: InfiniteListProps<T> & { ref?: Ref<InfiniteListHandle> },
) => ReactNode;
