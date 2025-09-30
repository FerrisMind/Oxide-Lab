# Автоматическое управление версиями в Oxide Lab

## Обзор

Проект настроен для автоматического считывания версии из файла `Cargo.toml` и синхронизации её во всех конфигурационных файлах.

## Как это работает

### 1. Источник истины - Cargo.toml

Версия определяется в файле `src-tauri/Cargo.toml`:

```toml
[package]
name = "oxide-lab"
version = "0.11.1"  # ← Единственное место, где нужно менять версию
```

### 2. Автоматическая синхронизация

- **tauri.conf.json**: автоматически синхронизируется через скрипт
- **package.json**: автоматически синхронизируется через скрипт

### 3. Скрипты автоматизации

#### `npm run sync-version`

Синхронизирует версию из `Cargo.toml` в `package.json` и `tauri.conf.json`

#### `npm run test-version-sync`

Проверяет, что все версии синхронизированы правильно

#### `npm run predev` / `npm run prebuild`

Автоматически запускают синхронизацию перед разработкой/сборкой

## Использование

### Обновление версии вручную

1. Измените версию в `src-tauri/Cargo.toml`
2. Запустите `npm run sync-version`
3. Версия автоматически обновится в `package.json`

### Автоматическое обновление версии

Используйте существующий скрипт `scripts/auto-version-bump.ps1`:

```powershell
# Автоматический бамп на основе conventional commits
.\scripts\auto-version-bump.ps1

# Ручной бамп
.\scripts\auto-version-bump.ps1 -Patch   # 0.11.1 → 0.11.2
.\scripts\auto-version-bump.ps1 -Minor   # 0.11.1 → 0.12.0
.\scripts\auto-version-bump.ps1 -Major   # 0.11.1 → 1.0.0
```

Скрипт автоматически:

- Обновляет версию в `Cargo.toml` через `cog`
- Синхронизирует версию в `package.json`
- Создает git тег
- Пушит изменения

### Проверка синхронизации

```bash
npm run test-version-sync
```

## Структура файлов

```
scripts/
├── auto-version-bump.ps1    # PowerShell скрипт для автоматического бампа
├── sync-version.cjs         # Node.js скрипт синхронизации версии
└── test-version-sync.cjs    # Тест синхронизации версий

src-tauri/
├── Cargo.toml               # Источник истины для версии
└── tauri.conf.json          # Автоматически синхронизируется

package.json                 # Автоматически синхронизируется
```

## Преимущества

1. **Единый источник истины**: версия определяется только в `Cargo.toml`
2. **Автоматическая синхронизация**: нет необходимости вручную обновлять несколько файлов
3. **Conventional Commits**: автоматический бамп на основе типа коммитов
4. **Тестирование**: проверка корректности синхронизации
5. **CI/CD готовность**: легко интегрируется в пайплайны сборки

## Troubleshooting

### Версии не синхронизированы

```bash
npm run sync-version
npm run test-version-sync
```

### Ошибка "version must be a semver string"

Убедитесь, что в `tauri.conf.json` указана корректная версия в формате SemVer (например, `"0.11.0"`), а не переменная окружения.

### Проблемы с PowerShell скриптом

Убедитесь, что установлен `cog`:

```bash
cargo install cog
```

## Примеры использования

### Сценарий 1: Новый фича-релиз

```bash
# 1. Сделать коммит с conventional commit
git commit -m "feat: добавить новую функцию экспорта"

# 2. Автоматический бамп (minor)
.\scripts\auto-version-bump.ps1
# Результат: 0.11.1 → 0.12.0
```

### Сценарий 2: Исправление бага

```bash
# 1. Сделать коммит с conventional commit
git commit -m "fix: исправить ошибку в обработке файлов"

# 2. Автоматический бамп (patch)
.\scripts\auto-version-bump.ps1
# Результат: 0.11.1 → 0.11.2
```

### Сценарий 3: Breaking changes

```bash
# 1. Сделать коммит с conventional commit
git commit -m "feat!: изменить API для работы с моделями

BREAKING CHANGE: изменен формат конфигурации моделей"

# 2. Автоматический бамп (major)
.\scripts\auto-version-bump.ps1
# Результат: 0.11.1 → 1.0.0
```
