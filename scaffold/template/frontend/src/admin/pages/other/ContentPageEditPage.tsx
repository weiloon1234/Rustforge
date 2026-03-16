import { useEffect, useMemo, useState } from "react";
import { ArrowLeft, Save } from "lucide-react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import type { AdminContentPageOutput } from "@admin/types";
import { CONTENT_PAGE_SYSTEM_FLAG } from "@admin/types";
import { api } from "@admin/api";
import { uploadAdminTiptapImage } from "@admin/tiptapUpload";
import type { ApiResponse } from "@shared/types";
import {
  Button,
  useLocaleStore,
  alertSuccess,
  attachmentUrl,
} from "@shared/components";
import { useAutoForm, type AutoFormDefaultValue } from "@shared/useAutoForm";

const CONTENT_PAGE_SYSTEM_YES = CONTENT_PAGE_SYSTEM_FLAG._1;

export default function ContentPageEditPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const params = useParams<{ id: string }>();
  const pageId = params.id ?? "";
  const availableLocales = useLocaleStore((s) => s.availableLocales);
  const defaultLocale = useLocaleStore((s) => s.defaultLocale);
  const locales = useMemo(
    () => (availableLocales.length > 0 ? availableLocales : [defaultLocale]),
    [availableLocales, defaultLocale],
  );

  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [contentPage, setContentPage] = useState<AdminContentPageOutput | null>(null);

  useEffect(() => {
    if (!pageId) {
      setLoading(false);
      setLoadError(t("Invalid page id"));
      return;
    }

    let active = true;
    setLoading(true);
    setLoadError(null);

    void api
      .get<ApiResponse<AdminContentPageOutput>>(`content_page/${pageId}`)
      .then((res) => {
        if (!active) return;
        const payload = res.data?.data;
        if (!payload) {
          setLoadError(t("Failed to load page detail."));
          setLoading(false);
          return;
        }
        setContentPage(payload);
        setLoading(false);
      })
      .catch((err) => {
        if (!active) return;
        const maybe = err as { response?: { data?: { message?: string } } };
        setLoadError(maybe?.response?.data?.message ?? t("Failed to load page detail."));
        setLoading(false);
      });

    return () => { active = false; };
  }, [pageId, t]);

  const isSystem = contentPage?.is_system === CONTENT_PAGE_SYSTEM_YES;

  const coverDefaults = useMemo(() => {
    if (!contentPage) return {};
    const result: Record<string, { name: string; url: string }> = {};
    for (const locale of locales) {
      const path = (contentPage.cover[locale] ?? "").trim();
      if (path) {
        result[locale] = {
          name: path,
          url: (contentPage.cover_url[locale] ?? "").trim() || attachmentUrl(path),
        };
      }
    }
    return result;
  }, [contentPage, locales]);

  const { submit, busy: saving, form, errors } = useAutoForm(api, {
    url: `content_page/${pageId}`,
    method: "patch",
    bodyType: "multipart",
    tiptapImageUpload: uploadAdminTiptapImage,
    fields: [
      { name: "tag", type: "text", label: t("Tag"), required: true, disabled: isSystem },
      {
        type: "localized",
        children: [
          { name: "title", type: "text", label: t("Title"), required: true, disabled: isSystem },
          { name: "content", type: "tiptap", label: t("Content"), required: true, editorPreset: "full", imageFolder: "uploads/content_page" } as any,
          { name: "cover", type: "file", label: t("Cover"), accept: "image/*", notes: t("Optional localized cover image") } as any,
        ],
      },
    ],
    defaults: contentPage ? {
      tag: contentPage.tag ?? "",
      title: contentPage.title ?? {},
      content: contentPage.content ?? {},
      cover: coverDefaults,
    } as unknown as Record<string, AutoFormDefaultValue> : undefined,
    onSuccess: () => {
      alertSuccess({ title: t("Success"), message: t("Page updated") });
      navigate("/other/content-pages");
    },
  });

  return (
    <section className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h1 className="text-2xl font-semibold">{t("Edit Page")}</h1>
          <p className="text-sm text-muted">{t("Update localized content and cover")}</p>
        </div>
        <Link
          to="/other/content-pages"
          className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-3 py-2 text-sm font-medium transition hover:bg-surface-hover"
        >
          <ArrowLeft size={16} />
          {t("Back to Pages")}
        </Link>
      </div>

      {loading && <p className="text-sm text-muted">{t("Loading…")}</p>}
      {!loading && loadError && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {loadError}
        </p>
      )}

      {!loading && contentPage && (
        <form onSubmit={submit} className="space-y-4">
          {errors.general && (
            <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
              {errors.general}
            </p>
          )}

          {form}

          <div className="sticky bottom-0 z-10 flex justify-end rounded-lg border border-border bg-background/95 px-4 py-3 backdrop-blur">
            <Button
              type="submit"
              busy={saving}
              variant="primary"
              size="sm"
            >
              <Save size={16} />
              {saving ? t("Saving…") : t("Save")}
            </Button>
          </div>
        </form>
      )}
    </section>
  );
}
