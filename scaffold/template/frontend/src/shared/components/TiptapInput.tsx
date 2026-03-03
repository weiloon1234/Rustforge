import { useEffect, useId, type ChangeEvent } from "react";
import { Bold, Italic, List, ListOrdered, Undo2, Redo2 } from "lucide-react";
import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";

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
}

function normalizeHtml(value: string): string {
  const trimmed = value.trim();
  if (!trimmed || trimmed === "<p></p>") return "";
  return trimmed;
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
}: TiptapInputProps) {
  const autoId = useId();
  const id = externalId ?? autoId;
  const hasError = hasFieldError(error, errors);
  const editor = useEditor({
    extensions: [StarterKit],
    content: typeof value === "string" ? value : "",
    editable: !disabled,
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

  return (
    <div className={`rf-field ${containerClassName ?? ""}`}>
      {label && (
        <label htmlFor={id} className={`rf-label ${required ? "rf-label-required" : ""}`}>
          {label}
        </label>
      )}
      <div className={`rf-rich ${hasError ? "rf-rich-error" : ""} ${className ?? ""}`}>
        <div className="rf-rich-toolbar">
          <button
            type="button"
            className={btnClass(!!editor?.isActive("bold"))}
            onClick={() => editor?.chain().focus().toggleBold().run()}
            disabled={!editor || disabled}
            title="Bold"
          >
            <Bold size={14} />
          </button>
          <button
            type="button"
            className={btnClass(!!editor?.isActive("italic"))}
            onClick={() => editor?.chain().focus().toggleItalic().run()}
            disabled={!editor || disabled}
            title="Italic"
          >
            <Italic size={14} />
          </button>
          <button
            type="button"
            className={btnClass(!!editor?.isActive("bulletList"))}
            onClick={() => editor?.chain().focus().toggleBulletList().run()}
            disabled={!editor || disabled}
            title="Bullet List"
          >
            <List size={14} />
          </button>
          <button
            type="button"
            className={btnClass(!!editor?.isActive("orderedList"))}
            onClick={() => editor?.chain().focus().toggleOrderedList().run()}
            disabled={!editor || disabled}
            title="Ordered List"
          >
            <ListOrdered size={14} />
          </button>
          <span className="rf-rich-divider" />
          <button
            type="button"
            className={btnClass(false)}
            onClick={() => editor?.chain().focus().undo().run()}
            disabled={!editor || disabled || !canUndo}
            title="Undo"
          >
            <Undo2 size={14} />
          </button>
          <button
            type="button"
            className={btnClass(false)}
            onClick={() => editor?.chain().focus().redo().run()}
            disabled={!editor || disabled || !canRedo}
            title="Redo"
          >
            <Redo2 size={14} />
          </button>
        </div>
        <EditorContent
          id={id}
          editor={editor}
          className={`rf-rich-editor ${disabled ? "rf-rich-disabled" : ""}`}
        />
      </div>
      <FieldErrors error={error} errors={errors} />
      {notes && !hasError && <p className="rf-note">{notes}</p>}
    </div>
  );
}

export const TapbitInput = TiptapInput;
