export { FieldErrors, hasFieldError } from "@shared/components/FieldErrors";
export type { FieldErrorsProps } from "@shared/components/FieldErrors";

export { TextInput } from "@shared/components/TextInput";
export type { TextInputProps } from "@shared/components/TextInput";

export { ContactInput } from "@shared/components/ContactInput";
export type { ContactInputProps, ContactInputValue } from "@shared/components/ContactInput";

export { TextArea } from "@shared/components/TextArea";
export type { TextAreaProps } from "@shared/components/TextArea";

export { Select } from "@shared/components/Select";
export type { SelectProps, SelectOption } from "@shared/components/Select";

export { Checkbox } from "@shared/components/Checkbox";
export type { CheckboxProps } from "@shared/components/Checkbox";

export { Button } from "@shared/components/Button";
export type { ButtonProps, ButtonVariant, ButtonSize } from "@shared/components/Button";

export { Radio } from "@shared/components/Radio";
export type { RadioProps, RadioOption } from "@shared/components/Radio";

export {
  DatePickerInput,
  DateTimePickerInput,
  TimePickerInput,
} from "@shared/components/TemporalInput";
export type { TemporalInputProps, TemporalInputType } from "@shared/components/TemporalInput";

export { TiptapInput, TapbitInput } from "@shared/components/TiptapInput";
export type {
  TiptapInputProps,
  TiptapImageUploadHandler,
  TiptapImageUploadResult,
  TiptapPreset,
} from "@shared/components/TiptapInput";

export { FileInput } from "@shared/components/FileInput";
export type { FileInputProps, FilePreviewItem } from "@shared/components/FileInput";

export { DataTable } from "@shared/components/DataTable";
export type {
  DataTableProps,
  DataTableColumn,
  DataTableCellContext,
  DataTablePreCallEvent,
  DataTablePostCallEvent,
  DataTableFooterContext,
  DataTableFilterSnapshot,
} from "@shared/components/DataTable";
export { DataTableApiProvider, useDataTableApi } from "@shared/components/DataTable";

export { useAutoForm } from "@shared/useAutoForm";
export type {
  FieldDef,
  AutoFormConfig,
  AutoFormErrors,
  AutoFormResult,
  AutoFormBodyType,
  AutoFormDefaultValue,
} from "@shared/useAutoForm";
export { useLocaleStore } from "@shared/stores/locale";
export { getRuntimeConfig } from "@shared/runtimeConfig";

export { Modal } from "@shared/components/Modal";
export { ModalOutlet } from "@shared/components/ModalOutlet";
export { useModalStore } from "@shared/useModalStore";
export type { ModalOptions, ModalSize, ModalEntry } from "@shared/useModalStore";

export {
  alertConfirm,
  alertSuccess,
  alertError,
  alertWarning,
  alertInfo,
  moneyFormat,
  formatDateTime,
  attachmentUrl,
} from "@shared/helpers";
