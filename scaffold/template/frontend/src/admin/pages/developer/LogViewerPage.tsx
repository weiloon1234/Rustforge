import { useCallback, useEffect, useState } from "react";
import { Trash2, RefreshCw, Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { LogFileEntry } from "@admin/types";
import { Button, Select, useModalStore } from "@shared/components";
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

const LOG_LEVELS = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"] as const;
type LogLevel = (typeof LOG_LEVELS)[number];

function getLineLevel(line: string): LogLevel | null {
  const match = line.match(/^\S+\s+(ERROR|WARN|INFO|DEBUG|TRACE)\s/);
  return match ? (match[1] as LogLevel) : null;
}

function highlightLine(line: string): React.ReactNode {
  const level = getLineLevel(line);
  if (!level) return line;
  return <span className={levelClass(level)}>{line}</span>;
}

function LogContent({
  content,
  searchQuery,
  minLevel,
}: {
  content: string;
  searchQuery: string;
  minLevel: string;
}) {
  const minIdx = LOG_LEVELS.indexOf(minLevel as LogLevel);
  const lines = content.split("\n").reverse();
  const filteredLines = lines.filter((line) => {
    if (minIdx > 0) {
      const level = getLineLevel(line);
      if (level && LOG_LEVELS.indexOf(level) < minIdx) return false;
    }
    if (searchQuery) {
      return line.toLowerCase().includes(searchQuery.toLowerCase());
    }
    return true;
  });

  return (
    <div className="flex-1 min-h-0">
      <pre className="max-h-[calc(100vh-16rem)] overflow-auto rounded-lg border border-border bg-gray-950 p-3 font-mono text-xs leading-5 text-gray-200">
        {filteredLines.map((line, i) => (
          <div key={i}>{highlightLine(line) || "\u00A0"}</div>
        ))}
      </pre>
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
  const [minLevel, setMinLevel] = useState("");

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

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
        <Select
          value={selectedFile}
          onChange={(e) => setSelectedFile(e.target.value)}
          options={files.map((f) => ({
            value: f.filename,
            label: `${f.filename} (${formatBytes(Number(f.size_bytes))})`,
          }))}
          placeholder={files.length === 0 ? t("No log files") : undefined}
          containerClassName="!mb-0 sm:min-w-[16rem] sm:max-w-[24rem]"
        />
        <div className="flex items-center gap-2">
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
        </div>
      </div>

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
        <Select
          value={minLevel}
          onChange={(e) => setMinLevel(e.target.value)}
          options={LOG_LEVELS.map((l) => ({ value: l, label: l }))}
          placeholder={t("All levels")}
          containerClassName="!mb-0 sm:min-w-[8rem] sm:max-w-[12rem]"
        />
        <div className="relative">
          <Search
            size={14}
            className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted"
          />
          <input
            type="text"
            placeholder={t("Filter logs...")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full h-8 rounded-lg border border-border bg-surface pl-8 pr-3 text-sm sm:w-auto"
          />
        </div>
      </div>

      {selectedMeta && (
        <div className="flex gap-4 text-xs text-muted">
          <span>
            {t("Size")}: {formatBytes(Number(selectedMeta.size_bytes))}
          </span>
          <span>
            {t("Lines")}: {content.split("\n").length.toLocaleString()}
          </span>
          {(searchQuery || minLevel) && (
            <span>
              {t("Showing")}:{" "}
              {content
                .split("\n")
                .filter((l) => {
                  if (minLevel) {
                    const lvl = getLineLevel(l);
                    if (lvl && LOG_LEVELS.indexOf(lvl) < LOG_LEVELS.indexOf(minLevel as LogLevel)) return false;
                  }
                  if (searchQuery) return l.toLowerCase().includes(searchQuery.toLowerCase());
                  return true;
                })
                .length.toLocaleString()}{" "}
              {t("matches")}
            </span>
          )}
        </div>
      )}

      <LogContent content={content} searchQuery={searchQuery} minLevel={minLevel} />
    </div>
  );
}
