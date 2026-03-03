import { Pencil, Trash2 } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import type { AdminPageDeleteOutput, PageDatatableRow, PageSystemFlag } from "@admin/types";
import { PAGE_SYSTEM_FLAG } from "@admin/types";
import { api } from "@admin/api";
import type { ApiResponse } from "@shared/types";
import {
  DataTable,
  alertConfirm,
  alertError,
  alertSuccess,
  formatDateTime,
} from "@shared/components";

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

function toSystemLabel(value: PageSystemFlag, t: (key: string) => string): string {
  if (value === PAGE_SYSTEM_FLAG.YES) return t("System");
  return t("Custom");
}

function toSystemBadgeClass(value: PageSystemFlag): string {
  if (value === PAGE_SYSTEM_FLAG.YES) return "bg-amber-100 text-amber-700";
  return "bg-emerald-100 text-emerald-700";
}

export default function PagesPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const handleDelete = async (row: PageDatatableRow, refresh: () => void) => {
    if (row.is_system === PAGE_SYSTEM_FLAG.YES) {
      return;
    }

    await alertConfirm({
      title: t("Delete Page"),
      message: t('Are you sure you want to delete ":tag"?', { tag: row.tag }),
      confirmText: t("Delete"),
      callback: async (result) => {
        if (!result.isConfirmed) return;
        try {
          await api.delete<ApiResponse<AdminPageDeleteOutput>>(`pages/${row.id}`);
          alertSuccess({ title: t("Deleted"), message: t("Page deleted") });
          refresh();
        } catch (err) {
          alertError({
            title: t("Error"),
            message: normalizeErrorMessage(err, t("Failed to delete page.")),
          });
        }
      },
    });
  };

  return (
    <DataTable<PageDatatableRow>
      url="datatable/page/query"
      title={t("Pages")}
      subtitle={t("Manage policy pages")}
      columns={[
        {
          key: "actions",
          label: t("Actions"),
          sortable: false,
          cellClassName: "text-foreground",
          render: (row, ctx) => (
            <div className="flex gap-1">
              <button
                type="button"
                onClick={() => navigate(`/pages/${row.id}/edit`)}
                className="rounded-lg p-1.5 text-muted transition hover:bg-surface-hover hover:text-foreground"
                title={t("Edit")}
              >
                <Pencil size={16} />
              </button>
              {row.is_system !== PAGE_SYSTEM_FLAG.YES && (
                <button
                  type="button"
                  onClick={() => handleDelete(row, ctx.refresh)}
                  className="rounded-lg p-1.5 text-muted transition hover:bg-red-50 hover:text-red-600"
                  title={t("Delete")}
                >
                  <Trash2 size={16} />
                </button>
              )}
            </div>
          ),
        },
        {
          key: "tag",
          label: t("Tag"),
          cellClassName: "font-medium text-foreground",
          render: (row) => row.tag,
        },
        {
          key: "title",
          label: t("Title"),
          cellClassName: "text-foreground",
          render: (row) => row.title ?? "—",
        },
        {
          key: "is_system",
          label: t("System"),
          cellClassName: "text-foreground",
          render: (row) => (
            <span
              className={`inline-block rounded-full px-2 py-0.5 text-xs font-medium ${toSystemBadgeClass(row.is_system)}`}
            >
              {toSystemLabel(row.is_system, t)}
            </span>
          ),
        },
        {
          key: "updated_at",
          label: t("Updated At"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => formatDateTime(row.updated_at),
        },
      ]}
    />
  );
}
