import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type ChangeEvent,
  type ReactNode,
} from "react";
import { Bold, Italic, List, ListOrdered, Undo2, Redo2 } from "lucide-react";
import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Link from "@tiptap/extension-link";
import Placeholder from "@tiptap/extension-placeholder";
import Underline from "@tiptap/extension-underline";
import Highlight from "@tiptap/extension-highlight";
import Image from "@tiptap/extension-image";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";
import Table from "@tiptap/extension-table";
import TableRow from "@tiptap/extension-table-row";
import TableHeader from "@tiptap/extension-table-header";
import TableCell from "@tiptap/extension-table-cell";
import CharacterCount from "@tiptap/extension-character-count";
import { useTranslation } from "react-i18next";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";
import { Button } from "@shared/components/Button";
import { alertError } from "@shared/helpers";
import {
  resolveTiptapPreset,
  type TiptapPreset as TiptapPresetName,
  type TiptapToolbarAction,
  type TiptapToolbarItem,
} from "@shared/components/tiptapPresets";

export type TiptapPreset = TiptapPresetName;

export type TiptapImageUploadResult = { url: string; path: string };
export type TiptapImageUploadHandler = (
  file: File,
  folder: string,
) => Promise<TiptapImageUploadResult>;

export interface TiptapInputProps {
  label?: string;
  value?: string;
  onChange?: (event: ChangeEvent<HTMLInputElement>) => void;
  error?: string;
  errors?: string[];
  notes?: string;
  required?: boolean;
  disabled?: boolean;
  containerClassName?: string;
  className?: string;
  id?: string;
  preset?: TiptapPreset;
  placeholder?: string;
  imageFolder?: string;
  imageUpload?: TiptapImageUploadHandler;
}

function normalizeHtml(value: string): string {
  const trimmed = value.trim();
  if (!trimmed || trimmed === "<p></p>") return "";
  return trimmed;
}

function normalizeLinkInput(rawInput: string): { url: string | null; invalid: boolean } {
  const trimmed = rawInput.trim();
  if (!trimmed) return { url: null, invalid: false };

  const candidate = /^https?:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`;

  try {
    const parsed = new URL(candidate);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
      return { url: null, invalid: true };
    }
    return { url: parsed.toString(), invalid: false };
  } catch {
    return { url: null, invalid: true };
  }
}

function resolveErrorMessage(error: unknown, fallback: string): string {
  const maybe = error as { response?: { data?: { message?: string } }; message?: string };
  return maybe?.response?.data?.message ?? maybe?.message ?? fallback;
}

export function TiptapInput({
  label,
  value,
  onChange,
  error,
  errors,
  notes,
  required,
  disabled,
  containerClassName,
  className,
  id: externalId,
  preset,
  placeholder,
  imageFolder,
  imageUpload,
}: TiptapInputProps) {
  const { t } = useTranslation();
  const autoId = useId();
  const id = externalId ?? autoId;
  const hasError = hasFieldError(error, errors);
  const [uploadingImage, setUploadingImage] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const presetConfig = resolveTiptapPreset(preset);
  const canUseImageUpload = !!imageUpload && !!(imageFolder ?? "").trim();

  const extensions = useMemo(() => {
    const nextExtensions = [
      StarterKit.configure({
        heading: { levels: [2, 3] },
      }),
      Placeholder.configure({
        placeholder: placeholder?.trim() || "",
      }),
    ];

    if (presetConfig.features.link) {
      nextExtensions.push(
        Link.configure({
          openOnClick: false,
          linkOnPaste: true,
          autolink: false,
          HTMLAttributes: {
            rel: "noopener noreferrer nofollow",
            target: "_blank",
          },
        }),
      );
    }
    if (presetConfig.features.underline) {
      nextExtensions.push(Underline);
    }
    if (presetConfig.features.highlight) {
      nextExtensions.push(Highlight);
    }
    if (presetConfig.features.image) {
      nextExtensions.push(Image.configure({ allowBase64: false }));
    }
    if (presetConfig.features.table) {
      nextExtensions.push(
        Table.configure({ resizable: false }),
        TableRow,
        TableHeader,
        TableCell,
      );
    }
    if (presetConfig.features.taskList) {
      nextExtensions.push(TaskList, TaskItem.configure({ nested: true }));
    }
    if (presetConfig.features.characterCount) {
      nextExtensions.push(CharacterCount);
    }

    return nextExtensions;
  }, [placeholder, presetConfig.features]);

  const editor = useEditor({
    extensions,
    content: typeof value === "string" ? value : "",
    editable: !disabled,
    editorProps: {
      transformPastedHTML: (html) =>
        html
          .replace(/<script[\s\S]*?>[\s\S]*?<\/script>/gi, "")
          .replace(/<style[\s\S]*?>[\s\S]*?<\/style>/gi, "")
          .replace(/<iframe[\s\S]*?>[\s\S]*?<\/iframe>/gi, ""),
    },
    onUpdate: ({ editor: nextEditor }) => {
      if (!onChange) return;
      const html = normalizeHtml(nextEditor.getHTML());
      const event = {
        target: { value: html },
        currentTarget: { value: html },
      } as unknown as ChangeEvent<HTMLInputElement>;
      onChange(event);
    },
  });

  useEffect(() => {
    if (!editor) return;
    editor.setEditable(!disabled);
  }, [editor, disabled]);

  useEffect(() => {
    if (!editor) return;
    const incoming = normalizeHtml(value ?? "");
    const current = normalizeHtml(editor.getHTML());
    if (incoming === current) return;
    editor.commands.setContent(incoming || "<p></p>", false);
  }, [editor, value]);

  const btnClass = (active: boolean) =>
    `rf-rich-btn ${active ? "rf-rich-btn-active" : ""}`;
  const canUndo = !!editor?.can().chain().focus().undo().run();
  const canRedo = !!editor?.can().chain().focus().redo().run();

  const runAction = async (action: TiptapToolbarAction) => {
    if (!editor || disabled) return;

    switch (action) {
      case "bold":
        editor.chain().focus().toggleBold().run();
        return;
      case "italic":
        editor.chain().focus().toggleItalic().run();
        return;
      case "strike":
        editor.chain().focus().toggleStrike().run();
        return;
      case "heading2":
        editor.chain().focus().toggleHeading({ level: 2 }).run();
        return;
      case "heading3":
        editor.chain().focus().toggleHeading({ level: 3 }).run();
        return;
      case "bulletList":
        editor.chain().focus().toggleBulletList().run();
        return;
      case "orderedList":
        editor.chain().focus().toggleOrderedList().run();
        return;
      case "blockquote":
        editor.chain().focus().toggleBlockquote().run();
        return;
      case "inlineCode":
        editor.chain().focus().toggleCode().run();
        return;
      case "codeBlock":
        editor.chain().focus().toggleCodeBlock().run();
        return;
      case "horizontalRule":
        editor.chain().focus().setHorizontalRule().run();
        return;
      case "link": {
        const previous = editor.getAttributes("link").href ?? "";
        const raw = window.prompt(t("Insert or edit link"), previous);
        if (raw === null) return;

        const normalized = normalizeLinkInput(raw);
        if (normalized.invalid) {
          await alertError({
            title: t("Error"),
            message: t("Invalid URL. Please use http or https."),
          });
          return;
        }

        if (!normalized.url) {
          editor.chain().focus().extendMarkRange("link").unsetLink().run();
          return;
        }

        editor
          .chain()
          .focus()
          .extendMarkRange("link")
          .setLink({
            href: normalized.url,
            target: "_blank",
            rel: "noopener noreferrer nofollow",
          })
          .run();
        return;
      }
      case "underline":
        editor.chain().focus().toggleUnderline().run();
        return;
      case "highlight":
        editor.chain().focus().toggleHighlight().run();
        return;
      case "image":
        if (!canUseImageUpload) {
          await alertError({
            title: t("Error"),
            message: t("Image upload is not configured."),
          });
          return;
        }
        fileInputRef.current?.click();
        return;
      case "table":
        if (editor.isActive("table")) {
          editor.chain().focus().deleteTable().run();
        } else {
          editor
            .chain()
            .focus()
            .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
            .run();
        }
        return;
      case "taskList":
        editor.chain().focus().toggleTaskList().run();
        return;
      case "undo":
        editor.chain().focus().undo().run();
        return;
      case "redo":
        editor.chain().focus().redo().run();
        return;
    }
  };

  const isActionActive = (action: TiptapToolbarAction): boolean => {
    if (!editor) return false;

    switch (action) {
      case "bold":
        return editor.isActive("bold");
      case "italic":
        return editor.isActive("italic");
      case "strike":
        return editor.isActive("strike");
      case "heading2":
        return editor.isActive("heading", { level: 2 });
      case "heading3":
        return editor.isActive("heading", { level: 3 });
      case "bulletList":
        return editor.isActive("bulletList");
      case "orderedList":
        return editor.isActive("orderedList");
      case "blockquote":
        return editor.isActive("blockquote");
      case "inlineCode":
        return editor.isActive("code");
      case "codeBlock":
        return editor.isActive("codeBlock");
      case "link":
        return editor.isActive("link");
      case "underline":
        return editor.isActive("underline");
      case "highlight":
        return editor.isActive("highlight");
      case "table":
        return editor.isActive("table");
      case "taskList":
        return editor.isActive("taskList");
      case "horizontalRule":
      case "image":
      case "undo":
      case "redo":
        return false;
    }
  };

  const isActionDisabled = (action: TiptapToolbarAction): boolean => {
    if (!editor || disabled) return true;
    if (action === "undo") return !canUndo;
    if (action === "redo") return !canRedo;
    if (action === "image" && (!canUseImageUpload || uploadingImage)) return true;
    return false;
  };

  const actionLabel = (action: TiptapToolbarAction): string => {
    switch (action) {
      case "bold":
        return t("Bold");
      case "italic":
        return t("Italic");
      case "strike":
        return t("Strikethrough");
      case "heading2":
        return t("Heading 2");
      case "heading3":
        return t("Heading 3");
      case "bulletList":
        return t("Bullet List");
      case "orderedList":
        return t("Ordered List");
      case "blockquote":
        return t("Blockquote");
      case "inlineCode":
        return t("Inline Code");
      case "codeBlock":
        return t("Code Block");
      case "horizontalRule":
        return t("Horizontal Rule");
      case "link":
        return t("Link");
      case "underline":
        return t("Underline");
      case "highlight":
        return t("Highlight");
      case "image":
        return t("Image");
      case "table":
        return t("Table");
      case "taskList":
        return t("Task List");
      case "undo":
        return t("Undo");
      case "redo":
        return t("Redo");
    }
  };

  const actionContent = (action: TiptapToolbarAction): ReactNode => {
    switch (action) {
      case "bold":
        return <Bold size={14} />;
      case "italic":
        return <Italic size={14} />;
      case "bulletList":
        return <List size={14} />;
      case "orderedList":
        return <ListOrdered size={14} />;
      case "undo":
        return <Undo2 size={14} />;
      case "redo":
        return <Redo2 size={14} />;
      case "strike":
        return <span className="rf-rich-btn-text">S</span>;
      case "heading2":
        return <span className="rf-rich-btn-text">H2</span>;
      case "heading3":
        return <span className="rf-rich-btn-text">H3</span>;
      case "blockquote":
        return <span className="rf-rich-btn-text">"</span>;
      case "inlineCode":
        return <span className="rf-rich-btn-text">&lt;/&gt;</span>;
      case "codeBlock":
        return <span className="rf-rich-btn-text">{"{ }"}</span>;
      case "horizontalRule":
        return <span className="rf-rich-btn-text">-</span>;
      case "link":
        return <span className="rf-rich-btn-text">URL</span>;
      case "underline":
        return <span className="rf-rich-btn-text">U</span>;
      case "highlight":
        return <span className="rf-rich-btn-text">HL</span>;
      case "image":
        return <span className="rf-rich-btn-text">IMG</span>;
      case "table":
        return <span className="rf-rich-btn-text">TB</span>;
      case "taskList":
        return <span className="rf-rich-btn-text">[]</span>;
    }
  };

  const toolbarItems = presetConfig.toolbar;

  const onImageSelected = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file || !editor || !imageUpload || disabled || !canUseImageUpload) return;

    const folder = (imageFolder ?? "").trim();

    setUploadingImage(true);
    try {
      const uploaded = await imageUpload(file, folder);
      if (!uploaded.url) {
        throw new Error(t("Upload failed."));
      }
      editor.chain().focus().setImage({ src: uploaded.url, alt: file.name }).run();
    } catch (err) {
      await alertError({
        title: t("Error"),
        message: resolveErrorMessage(err, t("Upload failed.")),
      });
    } finally {
      setUploadingImage(false);
    }
  };

  return (
    <div className={`rf-field ${containerClassName ?? ""}`}>
      {label && (
        <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </label>
      )}
      <div className={`rf-rich ${hasError ? "rf-rich-error" : ""} ${className ?? ""}`}>
        <div className="rf-rich-toolbar">
          {toolbarItems.map((item: TiptapToolbarItem, index: number) => {
            if (item === "divider") {
              return <span key={`divider-${index}`} className="rf-rich-divider" />;
            }
            if (item === "image" && !canUseImageUpload) {
              return null;
            }
            const labelText = actionLabel(item);
            return (
              <Button
                key={item}
                type="button"
                variant="plain"
                size="xs"
                iconOnly
                className={btnClass(isActionActive(item))}
                onClick={() => {
                  void runAction(item);
                }}
                disabled={isActionDisabled(item)}
                title={labelText}
                aria-label={labelText}
              >
                {actionContent(item)}
              </Button>
            );
          })}
        </div>
        <EditorContent
          id={id}
          editor={editor}
          className={`rf-rich-editor ${disabled ? "rf-rich-disabled" : ""}`}
        />
      </div>
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        className="sr-only"
        onChange={(event) => {
          void onImageSelected(event);
        }}
      />
      <FieldErrors error={error} errors={errors} />
      {notes && !hasError && <p className="rf-note">{notes}</p>}
    </div>
  );
}

export const TapbitInput = TiptapInput;
