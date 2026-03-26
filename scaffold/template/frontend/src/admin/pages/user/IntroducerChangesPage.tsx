import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import type {
  IntroducerChangeDatatableRow,
  AdminMeOutput,
} from "@admin/types";
import { PERMISSION } from "@admin/types";
import {
  Button,
  DataTable,
  useAutoForm,
  useModalStore,
  alertSuccess,
  alertError,
  formatDateTime,
  normalizeErrorMessage,
} from "@shared/components";
import { useAuthStore } from "@admin/stores/auth";
import { api } from "@admin/api";

function canChangeIntroducer(account: AdminMeOutput | null): boolean {
  if (!account) return false;
  return useAuthStore.hasPermission(PERMISSION.USER_CHANGE_INTRODUCER, account);
}

function ChangeIntroducerForm({
  formId,
  onBusyChange,
}: {
  formId: string;
  onBusyChange: (busy: boolean) => void;
}) {
  const { t } = useTranslation();
  const closeWithRefresh = useModalStore((s) => s.closeWithRefresh);

  const { submit, busy, form } = useAutoForm(api, {
    url: "introducer_changes",
    method: "post",
    fields: [
      {
        name: "user_username",
        type: "text",
        label: t("User Username"),
        placeholder: t("Enter user username"),
        required: true,
      },
      {
        name: "new_introducer_username",
        type: "text",
        label: t("New Introducer Username"),
        placeholder: t("Enter new introducer username"),
        required: true,
      },
      {
        name: "remark",
        type: "textarea",
        label: t("Remark"),
        placeholder: t("Enter remark"),
      },
    ],
    onSuccess: () => {
      closeWithRefresh();
      alertSuccess({ title: t("Success"), message: t("Introducer changed") });
    },
    onError: (error) => {
      alertError({
        title: t("Error"),
        message: normalizeErrorMessage(error, t("Failed to change introducer.")),
      });
    },
  });

  useEffect(() => {
    onBusyChange(busy);
  }, [busy, onBusyChange]);

  return (
    <form id={formId} onSubmit={submit}>
      {form}
    </form>
  );
}

export default function IntroducerChangesPage() {
  const { t } = useTranslation();
  const account = useAuthStore((state) => state.account);
  const canChange = canChangeIntroducer(account);

  const handleCreate = () => {
    const formId = `introducer-change-form-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    let modalId = "";
    const renderFooter = (busy: boolean) => (
      <>
        <Button
          type="button"
          onClick={() => useModalStore.getState().close()}
          variant="secondary"
          disabled={busy}
        >
          {t("Cancel")}
        </Button>
        <Button type="submit" form={formId} variant="primary" busy={busy}>
          {busy ? t("Submitting\u2026") : t("Submit")}
        </Button>
      </>
    );
    modalId = useModalStore.getState().open({
      title: t("Change Introducer"),
      size: "lg",
      content: (
        <ChangeIntroducerForm
          formId={formId}
          onBusyChange={(busy) => {
            if (!modalId) return;
            useModalStore
              .getState()
              .update(modalId, { footer: renderFooter(busy) });
          }}
        />
      ),
      footer: renderFooter(false),
    });
  };

  return (
    <DataTable<IntroducerChangeDatatableRow>
      url="datatable/introducer_change/query"
      title={t("Introducer Changes")}
      subtitle={t("View introducer change logs")}
      headerActions={
        canChange
          ? (
              <Button
                onClick={() => handleCreate()}
                variant="primary"
                size="sm"
              >
                <Plus size={16} />
                {t("Change Introducer")}
              </Button>
            )
          : undefined
      }
      columns={[
        {
          key: "user_username",
          label: t("User"),
          render: (row) => row.user_username ?? "\u2014",
        },
        {
          key: "from_username",
          label: t("From"),
          render: (row) => row.from_username ?? "\u2014",
        },
        {
          key: "to_username",
          label: t("To"),
          render: (row) => row.to_username ?? "\u2014",
        },
        {
          key: "admin_username",
          label: t("Admin"),
          cellClassName: "text-muted",
          render: (row) => row.admin_username ?? "\u2014",
        },
        {
          key: "remark",
          label: t("Remark"),
          cellClassName: "text-muted",
          render: (row) => row.remark ?? "\u2014",
        },
        {
          key: "created_at",
          label: t("Created At"),
          cellClassName: "tabular-nums text-muted",
          render: (row) => formatDateTime(row.created_at),
        },
      ]}
    />
  );
}
