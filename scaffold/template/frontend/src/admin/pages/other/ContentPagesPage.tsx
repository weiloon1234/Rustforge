import { Pencil, Trash2 } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import type { AdminContentPageDeleteOutput, ContentPageDatatableRow, ContentPageSystemFlag } from "@admin/types";
import { CONTENT_PAGE_SYSTEM_FLAG } from "@admin/types";
import { api } from "@admin/api";
import type { ApiResponse } from "@shared/types";
import {
  Button,
  DataTable,
  alertConfirm,
  alertError,
  alertSuccess,
  formatDateTime,
} from "@shared/components";

const CONTENT_PAGE_SYSTEM_YES: ContentPageSystemFlag = CONTENT_PAGE_SYSTEM_FLAG._1;

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

function toSystemLabel(value: ContentPageSystemFlag, t: (key: string) => string): string {
  if (value === CONTENT_PAGE_SYSTEM_YES) return t("System");
  return t("Custom");
}

function toSystemBadgeClass(value: ContentPageSystemFlag): string {
  if (value === CONTENT_PAGE_SYSTEM_YES) return "bg-amber-100 text-amber-700";
  return "bg-emerald-100 text-emerald-700";
}

export default function ContentPagesPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const handleDelete = async (row: ContentPageDatatableRow, refresh: () => void) => {
    if (row.is_system === CONTENT_PAGE_SYSTEM_YES) {
      return;
    }

    await alertConfirm({
      title: t("Delete Page"),
      message: t('Are you sure you want to delete ":tag"?', { tag: row.tag }),
      confirmText: t("Delete"),
      callback: async (result) => {
        if (!result.isConfirmed) return;
        try {
          await api.delete<ApiResponse<AdminContentPageDeleteOutput>>(
            `content_page/${row.id}`,
          );
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
    <DataTable<ContentPageDatatableRow>
      url="datatable/content_page/query"
      title={t("Pages")}
      subtitle={t("Manage policy pages")}
      columns={[
        {
          key: "actions",
          label: t("Actions"),
          sortable: false,
          render: (row, ctx) => (
            <div className="flex gap-1">
              <Button
                type="button"
                onClick={() => navigate(`/other/content-pages/${row.id}/edit`)}
                variant="plain"
                size="sm"
                iconOnly
                title={t("Edit")}
              >
                <Pencil size={16} />
              </Button>
              {row.is_system !== CONTENT_PAGE_SYSTEM_YES && (
                <Button
                  type="button"
                  onClick={() => handleDelete(row, ctx.refresh)}
                  variant="plain"
                  size="sm"
                  iconOnly
                  className="hover:bg-red-50 hover:text-red-600"
                  title={t("Delete")}
                >
                  <Trash2 size={16} />
                </Button>
              )}
            </div>
          ),
        },
        {
          key: "tag",
          label: t("Tag"),
          cellClassName: "font-medium",
          render: (row) => row.tag,
        },
        {
          key: "title",
          label: t("Title"),
          render: (row) => row.title ?? "—",
        },
        {
          key: "is_system",
          label: t("System"),
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
