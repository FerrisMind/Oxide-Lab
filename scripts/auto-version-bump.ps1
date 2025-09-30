Param(
    [string]$CogProfile = "tauri",
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

# 2) Обеспечить baseline-тег (Variant A)
$baseline = 'compliance-baseline'
$tagList = (git tag --list $baseline)
$hasBaseline = ($null -ne $tagList) -and ($tagList.Trim().Length -gt 0)
if ($InitBaseline -or -not $hasBaseline) {
    Exec "git tag -a $baseline -m \"baseline before conventional commits\""
}

# 3) Установить git hook для cog
try { Exec 'cog install-hook' } catch { Write-Host 'Skip hook install (already installed?)' -ForegroundColor DarkGray }

# 4) Проверка Conventional Commits после baseline
Write-Host "Checking commits from $baseline..HEAD" -ForegroundColor Gray
try {
    Exec "cog check $baseline..HEAD"
} catch {
    Write-Host "Non-compliant commits detected after $baseline. Auto bump may still work if compliant commits exist." -ForegroundColor Yellow
}

# 5) Выбор режима bump
$bumpCmd = $null
if ($Major) { $bumpCmd = "cog bump --major --profile $CogProfile" }
elseif ($Minor) { $bumpCmd = "cog bump --minor --profile $CogProfile" }
elseif ($Patch) { $bumpCmd = "cog bump --patch --profile $CogProfile" }
else { $bumpCmd = "cog bump --auto --profile $CogProfile" }

# 6) Выполнить bump
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

# 7) Показать версию и последний коммит
Write-Host ''
Write-Host 'Latest commit:' -ForegroundColor Gray
git log -1 --pretty=oneline

# 8) Push (если запрошено и был бамп)
if (-not $NoPush -and $bumpOk) {
    try { Exec 'git push' } catch { Write-Host 'Skip push' -ForegroundColor DarkGray }
    try { Exec 'git push --tags' } catch { Write-Host 'Skip tag push' -ForegroundColor DarkGray }
}

Write-Host 'Done.' -ForegroundColor Green


