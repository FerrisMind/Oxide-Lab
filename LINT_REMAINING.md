# Remaining linting / formatting tasks for Oxide-Lab

Этот документ описывает все оставшиеся задачи, которые нужно выполнить, чтобы привести проект к устойчивому состоянию линтинга и форматирования (ESLint, Stylelint, Prettier, CI, pre-commit hooks).

1. ESLint (TypeScript + Svelte 5)

- Проблемы, которые остались:
  - Неиспользуемые переменные (warn): надо пройтись по файлам и либо удалить/переименовать (`_name`), либо явно использовать/экспортировать.
  - Решить политику `@ts-ignore` / `@typescript-eslint/ban-ts-comment`: рекомендую в большинстве мест оставить `// @ts-ignore` только с комментарием почему.
- Шаги:
  - Запустить `npm run lint` и просмотреть все warnings.
  - Исправить вручную: удалить неиспользуемый код, переименовать параметры, или пометить `_`.
  - Если правило `ban-ts-comment` мешает — можно смягчить: в `eslintrc` поставить `"@typescript-eslint/ban-ts-comment": "warn"`.

2. Stylelint (CSS)

- Текущее состояние: конфиг временно relaxed, автопоправки применены.
- Дальнейшие шаги (поэтапно ужесточать):
  1. Включить `color-function-notation` (modern) и исправить все цвета (rgba -> rgb/hex или modern function) по проекту.
  2. Включить `keyframes-name-pattern` и привести имена keyframes к kebab-case.
  3. Включить `no-duplicate-selectors` и исправить дубли (объединить правила или убрать дубликаты).
  4. Включить `selector-pseudo-class-no-unknown` (с исключением `:global`) и поправить Svelte-specific селекторы.
- Как я могу помочь: включать по одному правилу и исправлять файлы автоматически/вручную.

3. Prettier

- Файл `.prettierrc.json` добавлен (singleQuote, trailingComma, printWidth).
- Шаги: запустить `npx prettier --write .` и закоммитить изменения (я могу сделать это по запросу).

4. CI и pre-commit hooks

- Добавить `lint-staged` + `husky`:
  - pre-commit: `lint-staged` с командами `eslint --fix` и `stylelint --fix` и `prettier --write` на соответствующие расширения.
- Добавить GitHub Actions workflow: `lint.yml` запускает `npm ci` и `npm run lint` / `npm run lint:fix` и `npx stylelint "src/**/*.css"`.

5. Полный порядок выполнения (рекомендуемый):

- A. Запустить `npm run lint` и `npx stylelint "src/**/*.css"` локально, собрать отчёты.
- B. Исправить все безопасные автопоправки (`--fix`).
- C. Ручная правка оставшихся предупреждений (я помогу автоматически и вручную).
- D. Запустить `prettier --write .`.
- E. Добавить Husky + lint-staged и CI workflow.

6. Я могу сделать это за вас автоматически сейчас. Пожалуйста подтвердите, какие шаги выполнить немедленно:

- [ ] Применить `eslint --fix` (уже выполнено частично).
- [ ] Применить `stylelint --fix` (уже выполнено частично).
- [ ] Запустить `prettier --write .` и закоммитить изменения.
- [ ] Добавить Husky + lint-staged и настроить pre-commit hook.
- [ ] Добавить GitHub Actions workflow `lint.yml`.

Если подтверждаете, поставьте галочки пунктов, которые хотите выполнить автоматически, или ответьте "всё" — и я начну.
