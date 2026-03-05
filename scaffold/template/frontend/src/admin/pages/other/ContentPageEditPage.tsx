import { useEffect, useMemo, useState } from "react";
import { ArrowLeft, Save } from "lucide-react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import type { AdminContentPageOutput, AdminContentPageUpdateOutput } from "@admin/types";
import { CONTENT_PAGE_SYSTEM_FLAG } from "@admin/types";
import { api } from "@admin/api";
import { uploadAdminTiptapImage } from "@admin/tiptapUpload";
import type { ApiResponse, LocaleCode } from "@shared/types";
import {
  Button,
  FileInput,
  TextInput,
  TiptapInput,
  useLocaleStore,
  alertSuccess,
  attachmentUrl,
} from "@shared/components";

const CONTENT_PAGE_SYSTEM_YES = CONTENT_PAGE_SYSTEM_FLAG._1;

function normalizeErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } } };
  return maybe?.response?.data?.message ?? fallback;
}

export default function ContentPageEditPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const params = useParams<{ id: string }>();
  const pageId = params.id ?? "";
  const availableLocales = useLocaleStore((s) => s.availableLocales);
  const defaultLocale = useLocaleStore((s) => s.defaultLocale);
  const locales = useMemo<LocaleCode[]>(
    () => (availableLocales.length > 0 ? availableLocales : [defaultLocale]),
    [availableLocales, defaultLocale],
  );

  const [busy, setBusy] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [contentPage, setContentPage] = useState<AdminContentPageOutput | null>(null);
  const [tag, setTag] = useState("");
  const [title, setTitle] = useState<Record<string, string>>({});
  const [content, setContent] = useState<Record<string, string>>({});
  const [cover, setCover] = useState<Record<string, string>>({});
  const [coverUrl, setCoverUrl] = useState<Record<string, string>>({});
  const [coverFiles, setCoverFiles] = useState<Record<string, File | null>>({});

  useEffect(() => {
    if (!pageId) {
      setBusy(false);
      setError(t("Invalid page id"));
      return;
    }

    let active = true;
    setBusy(true);
    setError(null);

    void api
      .get<ApiResponse<AdminContentPageOutput>>(`content_page/${pageId}`)
      .then((res) => {
        if (!active) return;
        const payload = res.data?.data;
        if (!payload) {
          setError(t("Failed to load page detail."));
          setBusy(false);
          return;
        }

        setContentPage(payload);
        setTag(payload.tag);

        const nextTitle: Record<string, string> = {};
        const nextContent: Record<string, string> = {};
        const nextCover: Record<string, string> = {};
        const nextCoverUrl: Record<string, string> = {};
        for (const locale of locales) {
          nextTitle[locale] = payload.title[locale] ?? "";
          nextContent[locale] = payload.content[locale] ?? "";
          nextCover[locale] = payload.cover[locale] ?? "";
          nextCoverUrl[locale] = payload.cover_url[locale] ?? "";
        }
        setTitle(nextTitle);
        setContent(nextContent);
        setCover(nextCover);
        setCoverUrl(nextCoverUrl);
        setBusy(false);
      })
      .catch((err) => {
        if (!active) return;
        setError(normalizeErrorMessage(err, t("Failed to load page detail.")));
        setBusy(false);
      });

    return () => {
      active = false;
    };
  }, [pageId, locales, t]);

  const isSystem = contentPage?.is_system === CONTENT_PAGE_SYSTEM_YES;

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    event.stopPropagation();
    if (!pageId || saving) return;

    setSaving(true);
    setError(null);

    try {
      const formData = new FormData();
      formData.append("tag", tag.trim());

      for (const locale of locales) {
        formData.append(`title.${locale}`, title[locale] ?? "");
        formData.append(`content.${locale}`, content[locale] ?? "");

        const selected = coverFiles[locale];
        if (selected) {
          formData.append(`cover.${locale}`, selected);
        } else {
          const existingPath = (cover[locale] ?? "").trim();
          if (existingPath) {
            formData.append(`cover.${locale}`, existingPath);
          }
        }
      }

      await api.patch<ApiResponse<AdminContentPageUpdateOutput>>(
        `content_page/${pageId}`,
        formData,
      );
      alertSuccess({ title: t("Success"), message: t("Page updated") });
      navigate("/other/content-pages");
    } catch (err) {
      setError(normalizeErrorMessage(err, t("Failed to update page.")));
    } finally {
      setSaving(false);
    }
  };

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

      {busy && <p className="text-sm text-muted">{t("Loading…")}</p>}
      {!busy && error && (
        <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
          {error}
        </p>
      )}

      {!busy && !error && (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="rounded-lg border border-border bg-surface px-4 py-4">
            <TextInput
              label={t("Tag")}
              value={tag}
              onChange={(e) => setTag(e.target.value)}
              disabled={isSystem || saving}
            />
          </div>

          {locales.map((locale) => (
            <div
              key={locale}
              className="space-y-3 rounded-lg border border-border bg-surface px-4 py-4"
            >
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">
                {t("Locale :locale", { locale })}
              </p>

              <TextInput
                label={t("Title")}
                value={title[locale] ?? ""}
                onChange={(e) =>
                  setTitle((prev) => ({ ...prev, [locale]: e.target.value }))
                }
                disabled={isSystem || saving}
              />

              <TiptapInput
                label={t("Content")}
                value={content[locale] ?? ""}
                onChange={(e) =>
                  setContent((prev) => ({ ...prev, [locale]: e.target.value }))
                }
                preset="full"
                imageFolder="uploads/content_page"
                imageUpload={uploadAdminTiptapImage}
                disabled={saving}
              />

              <FileInput
                label={t("Cover")}
                accept="image/*"
                multiple={false}
                files={coverFiles[locale] ? [coverFiles[locale] as File] : []}
                defaultFiles={
                  (cover[locale] ?? "").trim()
                    ? [
                        {
                          name: cover[locale],
                          url: (coverUrl[locale] ?? "").trim() || attachmentUrl(cover[locale]),
                        },
                      ]
                    : []
                }
                onChange={(e) => {
                  const next = e.target.files?.[0] ?? null;
                  setCoverFiles((prev) => ({ ...prev, [locale]: next }));
                }}
                disabled={saving}
                notes={t("Optional localized cover image")}
              />
            </div>
          ))}

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
