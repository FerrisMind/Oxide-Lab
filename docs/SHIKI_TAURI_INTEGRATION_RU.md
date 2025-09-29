Интеграция Shiki (ESM bundle) и локальная загрузка wasm в Tauri v2

Кратко:

- Используйте `shiki/bundle/web` для браузерного ESM-бандла или сгенерируйте кастомный бандл с `npx shiki-codegen`.
- В Tauri упакуйте сгенерированный бандл и wasm в assets и загружайте динамически через `import('./path/to/shiki.bundle.js')` или `import('shiki/bundle/web')` если используете пакет.

Рекомендованные шаги:

1. Установите shiki в проект (dev/deps):

```bash
npm install shiki --save
```

2. Генерация бандла (опционально, если нужно уменьшить размер):

```bash
npx shiki-codegen \
  --langs javascript,typescript,python,html,css,json \
  --themes github-light,github-dark \
  ./shiki.bundle.js
```

3. Поместите `shiki.bundle.js` и `onig.wasm`/`shiki.wasm` в `src-tauri/icons` или `src/assets/shiki/` (любая папка, которая будет добавлена в `tauri.conf.json` как ресурс).

4. В коде фронтенда (Svelte) загружайте динамически:

```ts
// @ts-ignore
const mod = await import('/assets/shiki/shiki.bundle.js');
const { createHighlighter } = mod;
const highlighter = await createHighlighter({ langs: ['javascript'], themes: ['github-dark'] });
```

5. Если используете официальный `shiki/bundle/web` из npm, можно динамически импортировать его без генерации:

```ts
// @ts-ignore
const mod = await import('shiki/bundle/web');
const { createHighlighter } = mod;
```

6. Внимание к WASM-движку (Oniguruma):

- При использовании `createHighlighterCore` нужно загрузить wasm отдельно через `import('shiki/wasm')` или поместить `onig.wasm` в ассеты и загрузить через `loadWasm()`.

7. Tauri packaging:

- Убедитесь, что папка с бандлом и wasm включена в `tauri.conf.json` `allowlist` и `embeddedAssets`/`resources`. Обычно добавляют в `tauri.conf.json` секцию `bundle.resources`.

Примечание: не меняйте файлы в `example/` — для сборки Tauri используйте свои настройки и поместите бандл в проектное `assets`.
