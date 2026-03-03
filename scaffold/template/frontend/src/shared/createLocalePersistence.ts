import type { AxiosInstance } from "axios";
import { useLocaleStore } from "@shared/stores/locale";
import type { LocaleCode } from "@shared/types/platform";

export interface LocaleAwareAccount {
  locale?: string | null;
}

interface LocaleUpdateResponse {
  data?: {
    locale?: string;
  };
}

export interface LocalePersistenceConfig<A extends LocaleAwareAccount> {
  api: AxiosInstance;
  updateEndpoint: string;
  getAccount: () => A | null;
  setAccount: (account: A | null) => void;
}

export interface LocaleChangeResult {
  ok: boolean;
  locale: LocaleCode;
  error?: unknown;
}

export interface LocalePersistence<A extends LocaleAwareAccount> {
  syncFromAccount: (account: A | null | undefined) => Promise<LocaleCode>;
  changeAndPersist: (nextLocale: LocaleCode) => Promise<LocaleChangeResult>;
}

export function createLocalePersistence<A extends LocaleAwareAccount>(
  config: LocalePersistenceConfig<A>,
): LocalePersistence<A> {
  return {
    syncFromAccount: async (account) => {
      return useLocaleStore.getState().syncFromAccountLocale(account?.locale ?? null);
    },
    changeAndPersist: async (nextLocale) => {
      const store = useLocaleStore.getState();
      const previous = store.locale;
      if (nextLocale === previous) {
        return { ok: true, locale: previous };
      }

      await store.setLocale(nextLocale);
      try {
        const response = await config.api.patch<LocaleUpdateResponse>(
          config.updateEndpoint,
          { locale: nextLocale },
        );
        const latestStore = useLocaleStore.getState();
        const savedLocale =
          latestStore.normalizeLocale(response.data?.data?.locale ?? nextLocale) ??
          nextLocale;
        if (savedLocale !== latestStore.locale) {
          await latestStore.setLocale(savedLocale);
        }

        const account = config.getAccount();
        if (account) {
          config.setAccount({ ...account, locale: savedLocale });
        }

        return { ok: true, locale: savedLocale };
      } catch (error) {
        await useLocaleStore.getState().setLocale(previous);
        return { ok: false, locale: previous, error };
      }
    },
  };
}
