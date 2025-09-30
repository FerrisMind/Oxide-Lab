Param(
    [string]$CogHookProfile = "tauri",
    [switch]$InitBaseline,
    [switch]$Minor,
    [switch]$Patch,
    [switch]$Major,
    [switch]$NoPush
)

function Exec {
    param([string]$Command)
    Write-Host "→ $Command" -ForegroundColor DarkCyan
    $oldEAP = $ErrorActionPreference
    $ErrorActionPreference = 'Stop'
    try {
        Invoke-Expression $Command
    } finally {
        $ErrorActionPreference = $oldEAP
    }
}

# 1) Проверка git-репозитория
Exec 'git rev-parse --is-inside-work-tree > $null'

# 2) Опционально создать baseline-тег (Variant A)
$baseline = 'compliance-baseline'
if ($InitBaseline) {
    $tagList = (git tag --list $baseline)
    $hasBaseline = ($null -ne $tagList) -and ($tagList.Trim().Length -gt 0)
    if (-not $hasBaseline) {
        Exec "git tag -a $baseline -m \"baseline before conventional commits\""
    } else {
        Write-Host "Baseline tag '$baseline' already exists, skipping." -ForegroundColor DarkGray
    }
}

# 3) Установить git hook для cog
try { Exec 'cog install-hook' } catch { Write-Host 'Skip hook install (already installed?)' -ForegroundColor DarkGray }

# 4) Проверка Conventional Commits после baseline
# 3.1) Если baseline существует, используем его для локального отчёта (не обязательно)
$tagList = (git tag --list $baseline)
$hasBaseline = ($null -ne $tagList) -and ($tagList.Trim().Length -gt 0)
if ($hasBaseline) {
    Write-Host "Checking commits from $baseline..HEAD" -ForegroundColor Gray
    try { Exec "cog check $baseline..HEAD" } catch { Write-Host "Non-compliant commits detected after $baseline." -ForegroundColor Yellow }
}

# 5) Если рабочее дерево грязное — временно прячем изменения (stash)
$dirty = ((git status --porcelain) | Measure-Object -Line).Lines -gt 0
$didStash = $false
if ($dirty) {
    Write-Host "Working tree is dirty – stashing changes (including untracked)..." -ForegroundColor Yellow
    Exec 'git stash push -u -m "pre-bump stash"'
    $didStash = $true
}

# 6) Выбор режима bump (используем --hook-profile для выбора профиля хуков)
$bumpCmd = $null
if ($Major) { $bumpCmd = "cog bump --major --hook-profile $CogHookProfile" }
elseif ($Minor) { $bumpCmd = "cog bump --minor --hook-profile $CogHookProfile" }
elseif ($Patch) { $bumpCmd = "cog bump --patch --hook-profile $CogHookProfile" }
else { $bumpCmd = "cog bump --auto --hook-profile $CogHookProfile" }

# 7) Выполнить bump
Write-Host "Bumping version: $bumpCmd" -ForegroundColor Green
$bumpOk = $true
try {
    Exec $bumpCmd
} catch {
    $msg = $_.Exception.Message
    if ($bumpCmd -like '*--auto*' -and ($msg -match 'No conventional commit' -or $msg -match 'No new commits')) {
        Write-Host 'Нет подходящих коммитов (feat/fix/BREAKING) — авто-бамп пропущен.' -ForegroundColor Yellow
        $bumpOk = $false
    } else {
        throw
    }
}

# 7.1) Синхронизировать версию в package.json после успешного бампа
if ($bumpOk) {
    Write-Host "Синхронизация версии в package.json..." -ForegroundColor Cyan
    try {
        Exec 'npm run sync-version'
    } catch {
        Write-Host 'Ошибка при синхронизации версии в package.json' -ForegroundColor Red
        throw
    }
}

# 8) Вернуть отложенные изменения, если были
if ($didStash) {
    Write-Host "Restoring stashed changes..." -ForegroundColor Yellow
    try { Exec 'git stash pop' } catch { Write-Host 'Stash pop reported conflicts or nothing to apply.' -ForegroundColor DarkYellow }
}

# 9) Показать версию и последний коммит
Write-Host ''
Write-Host 'Latest commit:' -ForegroundColor Gray
git log -1 --pretty=oneline

# 10) Push (если запрошено и был бамп)
if (-not $NoPush -and $bumpOk) {
    # Определяем текущую ветку и явно указываем её как целевую при пуше (устраняем конфликт с несколькими upstream)
    $currentBranch = (git rev-parse --abbrev-ref HEAD).Trim()
    try { Exec "git push -u origin $currentBranch" } catch { Write-Host 'Skip push' -ForegroundColor DarkGray }
    $headTags = (git tag --points-at HEAD) -split "\r?\n" | Where-Object { $_ -and ($_ -ne '$baseline') }
    foreach ($t in $headTags) {
        try { Exec "git push origin $t" } catch { Write-Host "Skip tag push: $t" -ForegroundColor DarkGray }
    }
}

Write-Host 'Done.' -ForegroundColor Green


