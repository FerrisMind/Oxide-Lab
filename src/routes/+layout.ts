// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const ssr = false;

import { loadTranslations, locale } from '$lib/i18n';
import { getSavedLocale } from '$lib/i18n';

/** @type {import('@sveltejs/kit').Load} */
export const load = async ({ url }) => {
  // Получаем сохраненную локаль из Store или backend (не определяем системную)
  const savedLocale = await getSavedLocale();
  
  // Используем сохраненную локаль или текущую из store, если она уже установлена
  const currentLocaleValue = locale.get();
  const localeToUse = savedLocale || currentLocaleValue || 'en';
  
  // Устанавливаем локаль, если она отличается от текущей
  if (currentLocaleValue !== localeToUse) {
    locale.set(localeToUse);
  }
  
  // Загружаем переводы для текущего пути с сохраненной локалью
  await loadTranslations(localeToUse, url.pathname);
  
  return {};
};
