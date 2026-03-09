import { useCallback, useEffect, useRef, useState } from "react";
import { Trash2, RefreshCw, Search, ArrowDown } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { LogFileEntry } from "@admin/types";
import { Button, useModalStore } from "@shared/components";
import { useAuthStore } from "@admin/stores/auth";
import axios from "axios";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function levelClass(level: string): string {
  switch (level) {
    case "ERROR":
      return "text-red-400";
    case "WARN":
      return "text-amber-400";
    case "INFO":
      return "text-sky-400";
    case "DEBUG":
      return "text-gray-400";
    case "TRACE":
      return "text-gray-500";
    default:
      return "text-gray-300";
  }
}

function highlightLine(line: string): React.ReactNode {
  const match = line.match(/^\S+\s+(ERROR|WARN|INFO|DEBUG|TRACE)\s/);
  if (!match) return line;
  const level = match[1];
  return <span className={levelClass(level)}>{line}</span>;
}

function LogContent({
  content,
  searchQuery,
}: {
  content: string;
  searchQuery: string;
}) {
  const containerRef = useRef<HTMLPreElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [content, autoScroll]);

  const handleScroll = () => {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    setAutoScroll(scrollHeight - scrollTop - clientHeight < 50);
  };

  const scrollToBottom = () => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
      setAutoScroll(true);
    }
  };

  const lines = content.split("\n");
  const filteredLines = searchQuery
    ? lines.filter((line) =>
        line.toLowerCase().includes(searchQuery.toLowerCase()),
      )
    : lines;

  return (
    <div className="relative flex-1">
      <pre
        ref={containerRef}
        onScroll={handleScroll}
        className="h-full overflow-auto rounded-lg border border-border bg-gray-950 p-3 font-mono text-xs leading-5 text-gray-200"
      >
        {filteredLines.map((line, i) => (
          <div key={i}>{highlightLine(line) || "\u00A0"}</div>
        ))}
      </pre>
      {!autoScroll && (
        <button
          type="button"
          onClick={scrollToBottom}
          className="absolute bottom-4 right-4 rounded-full bg-primary p-2 text-white shadow-lg transition-opacity hover:opacity-80"
          title="Scroll to bottom"
        >
          <ArrowDown size={16} />
        </button>
      )}
    </div>
  );
}

export default function LogViewerPage() {
  const { t } = useTranslation();
  const [files, setFiles] = useState<LogFileEntry[]>([]);
  const [selectedFile, setSelectedFile] = useState<string>("");
  const [content, setContent] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  const token = useAuthStore((s) => s.token);

  const apiHeaders = useCallback(
    () => ({ Authorization: `Bearer ${token}` }),
    [token],
  );

  const fetchFiles = useCallback(async () => {
    try {
      const res = await axios.get("/api/v1/admin/developer/logs", {
        headers: apiHeaders(),
      });
      const list: LogFileEntry[] = res.data?.data?.files ?? [];
      setFiles(list);
      if (!selectedFile && list.length > 0) {
        setSelectedFile(list[0].filename);
      }
    } catch {
      setFiles([]);
    }
  }, [apiHeaders, selectedFile]);

  const fetchContent = useCallback(
    async (filename: string) => {
      if (!filename) return;
      setLoading(true);
      try {
        const res = await axios.get(
          `/api/v1/admin/developer/logs/${encodeURIComponent(filename)}`,
          { headers: apiHeaders() },
        );
        setContent(res.data?.data ?? "");
      } catch {
        setContent("Failed to load log file.");
      } finally {
        setLoading(false);
      }
    },
    [apiHeaders],
  );

  useEffect(() => {
    fetchFiles();
  }, [fetchFiles]);

  useEffect(() => {
    if (selectedFile) {
      fetchContent(selectedFile);
    }
  }, [selectedFile, fetchContent]);

  const handleDelete = (filename: string) => {
    useModalStore.getState().open({
      title: t("Delete Log File"),
      size: "sm",
      content: (
        <p className="text-sm">
          {t("Are you sure you want to delete")} <strong>{filename}</strong>?
        </p>
      ),
      footer: (
        <div className="flex gap-2">
          <Button
            type="button"
            onClick={async () => {
              try {
                await axios.delete(
                  `/api/v1/admin/developer/logs/${encodeURIComponent(filename)}`,
                  { headers: apiHeaders() },
                );
                useModalStore.getState().close();
                if (selectedFile === filename) {
                  setSelectedFile("");
                  setContent("");
                }
                fetchFiles();
              } catch {
                // error handled by axios interceptor
              }
            }}
            variant="danger"
            size="sm"
          >
            {t("Delete")}
          </Button>
          <Button
            type="button"
            onClick={() => useModalStore.getState().close()}
            variant="secondary"
            size="sm"
          >
            {t("Cancel")}
          </Button>
        </div>
      ),
    });
  };

  const selectedMeta = files.find((f) => f.filename === selectedFile);

  return (
    <div className="flex h-full flex-col gap-4 p-4">
      <div>
        <h1 className="text-lg font-semibold">{t("Log Viewer")}</h1>
        <p className="text-sm text-muted">
          {t("View and manage application log files")}
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <select
          value={selectedFile}
          onChange={(e) => setSelectedFile(e.target.value)}
          className="rounded-lg border border-border bg-surface px-3 py-1.5 text-sm"
        >
          {files.length === 0 && (
            <option value="">{t("No log files")}</option>
          )}
          {files.map((f) => (
            <option key={f.filename} value={f.filename}>
              {f.filename} ({formatBytes(f.size_bytes)})
            </option>
          ))}
        </select>

        <Button
          type="button"
          onClick={() => {
            fetchFiles();
            if (selectedFile) fetchContent(selectedFile);
          }}
          variant="secondary"
          size="sm"
          title={t("Refresh")}
        >
          <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
          <span className="ml-1">{t("Refresh")}</span>
        </Button>

        {selectedFile && (
          <Button
            type="button"
            onClick={() => handleDelete(selectedFile)}
            variant="danger"
            size="sm"
            title={t("Delete")}
          >
            <Trash2 size={14} />
            <span className="ml-1">{t("Delete")}</span>
          </Button>
        )}

        <div className="relative ml-auto">
          <Search
            size={14}
            className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted"
          />
          <input
            type="text"
            placeholder={t("Filter logs...")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="rounded-lg border border-border bg-surface py-1.5 pl-8 pr-3 text-sm"
          />
        </div>
      </div>

      {selectedMeta && (
        <div className="flex gap-4 text-xs text-muted">
          <span>
            {t("Size")}: {formatBytes(selectedMeta.size_bytes)}
          </span>
          <span>
            {t("Lines")}: {content.split("\n").length.toLocaleString()}
          </span>
          {searchQuery && (
            <span>
              {t("Showing")}:{" "}
              {content
                .split("\n")
                .filter((l) =>
                  l.toLowerCase().includes(searchQuery.toLowerCase()),
                )
                .length.toLocaleString()}{" "}
              {t("matches")}
            </span>
          )}
        </div>
      )}

      <LogContent content={content} searchQuery={searchQuery} />
    </div>
  );
}
