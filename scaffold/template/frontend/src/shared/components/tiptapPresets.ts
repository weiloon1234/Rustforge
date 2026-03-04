export type TiptapPreset = "basic" | "content" | "full";

export type TiptapToolbarAction =
  | "bold"
  | "italic"
  | "strike"
  | "heading2"
  | "heading3"
  | "bulletList"
  | "orderedList"
  | "blockquote"
  | "inlineCode"
  | "codeBlock"
  | "horizontalRule"
  | "link"
  | "underline"
  | "highlight"
  | "image"
  | "table"
  | "taskList"
  | "undo"
  | "redo";

export type TiptapToolbarItem = TiptapToolbarAction | "divider";

export interface TiptapPresetConfig {
  toolbar: readonly TiptapToolbarItem[];
  features: {
    link: boolean;
    underline: boolean;
    highlight: boolean;
    image: boolean;
    table: boolean;
    taskList: boolean;
    characterCount: boolean;
  };
}

const TIPTAP_PRESETS: Record<TiptapPreset, TiptapPresetConfig> = {
  basic: {
    toolbar: [
      "bold",
      "italic",
      "strike",
      "heading2",
      "heading3",
      "bulletList",
      "orderedList",
      "blockquote",
      "inlineCode",
      "codeBlock",
      "horizontalRule",
      "divider",
      "undo",
      "redo",
    ],
    features: {
      link: false,
      underline: false,
      highlight: false,
      image: false,
      table: false,
      taskList: false,
      characterCount: false,
    },
  },
  content: {
    toolbar: [
      "bold",
      "italic",
      "strike",
      "heading2",
      "heading3",
      "bulletList",
      "orderedList",
      "blockquote",
      "inlineCode",
      "codeBlock",
      "horizontalRule",
      "link",
      "underline",
      "highlight",
      "image",
      "divider",
      "undo",
      "redo",
    ],
    features: {
      link: true,
      underline: true,
      highlight: true,
      image: true,
      table: false,
      taskList: false,
      characterCount: false,
    },
  },
  full: {
    toolbar: [
      "bold",
      "italic",
      "strike",
      "heading2",
      "heading3",
      "bulletList",
      "orderedList",
      "blockquote",
      "inlineCode",
      "codeBlock",
      "horizontalRule",
      "link",
      "underline",
      "highlight",
      "image",
      "table",
      "taskList",
      "divider",
      "undo",
      "redo",
    ],
    features: {
      link: true,
      underline: true,
      highlight: true,
      image: true,
      table: true,
      taskList: true,
      characterCount: true,
    },
  },
};

export function resolveTiptapPreset(preset?: TiptapPreset): TiptapPresetConfig {
  return TIPTAP_PRESETS[preset ?? "basic"];
}
