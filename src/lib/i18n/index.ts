/**
 * i18n Initialization and Storage Module
 *
 * Модуль инициализации i18n и управления сохранением выбранной локали.
 * Использует Tauri Store для персистентного хранения предпочтений пользователя.
 */

import { loadTranslations, locale } from './config';
import { Store } from '@tauri-apps/plugin-store';

// Ключ для хранения локали в Tauri Store
const LOCALE_STORE_KEY = 'app.locale';

// Поддерживаемые локали
export const SUPPORTED_LOCALES = ['en', 'ru', 'pt-BR'] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

// Store для сохранения настроек
let store: Store | null = null;

/**
 * Инициализация Tauri Store для сохранения локали
 */
async function initStore(): Promise<Store> {
  if (!store) {
    store = await Store.load('.settings.dat');
  }
  return store;
}

/**
 * Определение системной локали на основе navigator.language
 * @returns Определенная локаль или 'en' по умолчанию
 */
function detectSystemLocale(): SupportedLocale {
  if (typeof navigator === 'undefined') {
    return 'en';
  }

  const systemLang = navigator.language || navigator.languages?.[0] || 'en';

  // Проверяем точное совпадение
  if (SUPPORTED_LOCALES.includes(systemLang as SupportedLocale)) {
    return systemLang as SupportedLocale;
  }

  // Проверяем префикс языка (например, 'ru-RU' -> 'ru')
  const langPrefix = systemLang.split('-')[0];
  if (SUPPORTED_LOCALES.includes(langPrefix as SupportedLocale)) {
    return langPrefix as SupportedLocale;
  }

  // Проверяем специальные случаи для португальского
  if (systemLang.startsWith('pt')) {
    return 'pt-BR';
  }

  return 'en';
}

/**
 * Получение сохраненной локали из Store или backend (без определения системной)
 * Используется при переключении вкладок, чтобы не сбрасывать язык на системный
 * @returns Сохраненная локаль или null, если не найдена
 */
export async function getSavedLocale(): Promise<SupportedLocale | null> {
  // Сначала пытаемся получить из backend (Rust)
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const backendLocale = await invoke<string>('get_locale');
    if (backendLocale && SUPPORTED_LOCALES.includes(backendLocale as SupportedLocale)) {
      if (import.meta.env.DEV) {
        console.log(`[i18n] Using locale from backend: ${backendLocale}`);
      }
      return backendLocale as SupportedLocale;
    }
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn('[i18n] Failed to load locale from backend:', error);
    }
  }

  // Затем пытаемся получить из Store
  try {
    const storeInstance = await initStore();
    const savedLocale = await storeInstance.get<SupportedLocale>(LOCALE_STORE_KEY);

    if (savedLocale && SUPPORTED_LOCALES.includes(savedLocale)) {
      if (import.meta.env.DEV) {
        console.log(`[i18n] Using locale from store: ${savedLocale}`);
      }
      return savedLocale;
    }
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn('[i18n] Failed to load saved locale from store:', error);
    }
  }

  return null;
}

/**
 * Получение сохраненной локали из Store или определение системной
 * Используется только при первой инициализации приложения
 * @returns Локаль для использования
 */
export async function getInitialLocale(): Promise<SupportedLocale> {
  // Сначала пытаемся получить сохраненную локаль
  const savedLocale = await getSavedLocale();
  if (savedLocale) {
    return savedLocale;
  }

  // Если сохраненной локали нет, определяем системную (только при первой загрузке)
  const systemLocale = detectSystemLocale();
  if (import.meta.env.DEV) {
    console.log(`[i18n] No saved locale found, using system locale: ${systemLocale}`);
  }
  return systemLocale;
}

/**
 * Сохранение выбранной локали в Store
 * @param locale - Локаль для сохранения
 */
export async function saveLocale(locale: SupportedLocale): Promise<void> {
  try {
    const storeInstance = await initStore();
    await storeInstance.set(LOCALE_STORE_KEY, locale);
    await storeInstance.save();
  } catch (error) {
    console.error('[i18n] Failed to save locale to store:', error);
  }
}

/**
 * Установка локали и загрузка переводов
 * @param newLocale - Новая локаль
 * @param pathname - Текущий путь (для загрузки route-specific переводов)
 */
export async function setLocale(newLocale: SupportedLocale, pathname: string = '/'): Promise<void> {
  if (!SUPPORTED_LOCALES.includes(newLocale)) {
    console.warn(`[i18n] Unsupported locale: ${newLocale}, falling back to 'en'`);
    newLocale = 'en';
  }

  // Устанавливаем локаль в sveltekit-i18n
  locale.set(newLocale);

  // Загружаем переводы для текущего пути
  // loadTranslations автоматически загрузит все необходимые переводы на основе routes
  // и общие переводы (common, sidebar, errors, about) без routes
  try {
    if (import.meta.env.DEV) {
      console.log(`[i18n] Loading translations for locale: ${newLocale}, path: ${pathname}`);
    }
    await loadTranslations(newLocale, pathname);
    if (import.meta.env.DEV) {
      console.log(`[i18n] Translations loaded successfully for locale: ${newLocale}`);
    }
  } catch (error) {
    console.error(
      `[i18n] Failed to load translations for locale ${newLocale} and path ${pathname}:`,
      error,
    );
    // Пробуем загрузить с fallback локалью
    if (newLocale !== 'en') {
      console.warn(`[i18n] Falling back to English translations`);
      await loadTranslations('en', pathname);
    }
  }

  // Сохраняем выбор пользователя
  await saveLocale(newLocale);
}

/**
 * Инициализация i18n при загрузке приложения
 * @param pathname - Текущий путь
 */
export async function initI18n(pathname: string = '/'): Promise<void> {
  const initialLocale = await getInitialLocale();
  await setLocale(initialLocale, pathname);
}

/**
 * Синхронизация локали с backend (Rust)
 * @param locale - Локаль для синхронизации
 */
export async function syncLocaleWithBackend(locale: SupportedLocale): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('set_locale', { locale });
  } catch (error) {
    console.warn('[i18n] Failed to sync locale with backend:', error);
  }
}

// Экспортируем функции из config для удобства
export { t, locale, locales, loading, loadTranslations } from './config';
