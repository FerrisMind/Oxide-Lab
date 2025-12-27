/**
 * i18n Initialization and Storage Module
 *
 * Модуль инициализации i18n и управления сохранением выбранной локали.
 * Использует Tauri Store для персистентного хранения предпочтений пользователя.
 */

import { loadTranslations, locale } from './config';

// Ключ для хранения локали в localStorage (fallback)
const LOCALE_STORAGE_KEY = 'oxide-locale';

// Поддерживаемые локали
export const SUPPORTED_LOCALES = ['en', 'ru', 'pt-BR'] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

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
 * Получение сохраненной локали из Store или backend
 * @returns Сохраненная локаль или null, если не найдена
 */
export async function getSavedLocale(): Promise<SupportedLocale | null> {
    // TODO: Integrate with Tauri backend
    // Command: invoke('get_locale')
    // Expected response: string (locale code)

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

    // Fallback to localStorage
    try {
        if (typeof localStorage !== 'undefined') {
            const savedLocale = localStorage.getItem(LOCALE_STORAGE_KEY);
            if (savedLocale && SUPPORTED_LOCALES.includes(savedLocale as SupportedLocale)) {
                if (import.meta.env.DEV) {
                    console.log(`[i18n] Using locale from localStorage: ${savedLocale}`);
                }
                return savedLocale as SupportedLocale;
            }
        }
    } catch (error) {
        if (import.meta.env.DEV) {
            console.warn('[i18n] Failed to load saved locale from localStorage:', error);
        }
    }

    return null;
}

/**
 * Получение сохраненной локали или определение системной
 * @returns Локаль для использования
 */
export async function getInitialLocale(): Promise<SupportedLocale> {
    const savedLocale = await getSavedLocale();
    if (savedLocale) {
        return savedLocale;
    }

    const systemLocale = detectSystemLocale();
    if (import.meta.env.DEV) {
        console.log(`[i18n] No saved locale found, using system locale: ${systemLocale}`);
    }
    return systemLocale;
}

/**
 * Сохранение выбранной локали
 * @param localeCode - Локаль для сохранения
 */
export async function saveLocale(localeCode: SupportedLocale): Promise<void> {
    // Save to localStorage as fallback
    try {
        if (typeof localStorage !== 'undefined') {
            localStorage.setItem(LOCALE_STORAGE_KEY, localeCode);
        }
    } catch (error) {
        console.error('[i18n] Failed to save locale to localStorage:', error);
    }

    // TODO: Integrate with Tauri backend
    // Command: invoke('set_locale', { locale: localeCode })
    try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('set_locale', { locale: localeCode });
    } catch (error) {
        if (import.meta.env.DEV) {
            console.warn('[i18n] Failed to sync locale with backend:', error);
        }
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

// Экспортируем функции из config для удобства
export { t, locale, locales, loading, loadTranslations } from './config';
