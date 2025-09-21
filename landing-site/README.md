# Oxide Lab Landing

Отдельный лендинг-проект на SvelteKit (Svelte 5) с anime.js анимациями. Проект собирается и деплоится независимо от основного приложения.

## Быстрый старт

```bash
cd landing-site
npm install
npm run dev
```

## Сборка (статический адаптер)

```bash
npm run build
```

В результате работы `@sveltejs/adapter-static` готовый сайт помещается в директорию `build/`, а файл `index.html` используется как SPA fallback. Чтобы посмотреть результат локально, выполните:

```bash
npm run preview
```

## Структура

- `src/components` — функциональные блоки лендинга (герой, возможности, сценарий, призыв к действию)
- `src/styles` — глобальные стили лендинга
- `src/routes/+page.svelte` — входная страница SvelteKit
- `src/app.html` — HTML-шаблон приложения

Проект использует руны Svelte 5 (`$state`, `$derived`, `$effect`) и анимации anime.js. Для появления блоков при скролле подключена директория `reveal` на базе `IntersectionObserver`.
