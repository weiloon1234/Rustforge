export type LocaleCode = "en" | "zh";
export const DEFAULT_LOCALE: LocaleCode = "en";

// Localized text payload generated from app language settings.
export type MultiLang<TLocale extends string = LocaleCode> = Record<TLocale, string>;

// field -> owner_id -> locale -> value
export type LocalizedMap<TLocale extends string = LocaleCode> = Record<
  string,
  Record<number, Record<TLocale, string>>
>;

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | JsonValue[];
export interface JsonObject {
  [key: string]: JsonValue;
}

// Generic typed meta shape (compile-time typed keys/values).
export type MetaRecord<
  TShape extends Record<string, unknown> = Record<string, JsonValue>
> = Partial<TShape>;

// field -> owner_id -> value
export type MetaMap = Record<string, Record<number, JsonValue>>;

export interface AttachmentUploadDto {
  id?: string | null;
  name?: string | null;
  path: string;
  content_type: string;
  size: number;
  width?: number | null;
  height?: number | null;
}

// Backward-compatible alias used by generated model APIs.
export type AttachmentInput = AttachmentUploadDto;

export interface Attachment {
  id: string;
  path: string;
  content_type: string;
  size: number;
  width: number | null;
  height: number | null;
  created_at: string;
}

// field -> owner_id -> attachments
export type AttachmentMap = Record<string, Record<number, Attachment[]>>;
