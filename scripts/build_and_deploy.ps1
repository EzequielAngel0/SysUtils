param(
    [switch]$SkipBuild,
    [switch]$NoShortcut
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$NativeDir   = $ProjectRoot
$TargetDir   = Join-Path $NativeDir "target\release"
$ExeName     = "sysutils_native.exe"
$AppName     = "SysUtils"
$IconSource  = Join-Path $NativeDir "assets\icon.ico"

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host "         SYSUTILS - Build & Deploy        " -ForegroundColor Cyan
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host ""

# -- Step 1: Build ----------------------------------------------------------------
if (-not $SkipBuild) {
    Write-Host "[1/3] Compilando en modo Release..." -ForegroundColor Cyan

    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("Path","User")

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "  [!] Rust/Cargo no detectado. Instala Rust desde https://rustup.rs/" -ForegroundColor Red
        exit 1
    }

    Push-Location $NativeDir
    try {
        $env:RUSTFLAGS = "-C target-cpu=native"
        cargo build --release 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  [X] Error de compilacion." -ForegroundColor Red
            Pop-Location
            exit 1
        }
        Write-Host "  [OK] Compilacion exitosa." -ForegroundColor Green
    } finally {
        Pop-Location
    }
} else {
    Write-Host "[1/3] Compilacion omitida (--SkipBuild)." -ForegroundColor Yellow
}

# -- Step 2: Verify executable ----------------------------------------------------
$ExePath = Join-Path $TargetDir $ExeName

if (-not (Test-Path $ExePath)) {
    Write-Host "  [X] No se encontro $ExePath" -ForegroundColor Red
    exit 1
}

$SizeMB = [math]::Round((Get-Item $ExePath).Length / 1MB, 2)
Write-Host "[2/3] Ejecutable listo: $ExePath ($SizeMB MB)" -ForegroundColor Green

# -- Step 3: Create Desktop Shortcut ----------------------------------------------
if (-not $NoShortcut) {
    Write-Host "[3/3] Creando accesos directos..." -ForegroundColor Cyan

    $DesktopPath   = [Environment]::GetFolderPath("Desktop")
    $StartMenuPath = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"

    $WshShell = New-Object -ComObject WScript.Shell

    foreach ($LnkPath in @((Join-Path $DesktopPath "$AppName.lnk"), (Join-Path $StartMenuPath "$AppName.lnk"))) {
        $Shortcut = $WshShell.CreateShortcut($LnkPath)
        $Shortcut.TargetPath       = $ExePath
        $Shortcut.WorkingDirectory = $TargetDir
        $Shortcut.Description      = "SysUtils — Hardware Automation Suite"
        $Shortcut.WindowStyle      = 1
        if (Test-Path $IconSource) { $Shortcut.IconLocation = $IconSource }
        $Shortcut.Save()
        Write-Host "  [OK] $LnkPath" -ForegroundColor Green
    }
} else {
    Write-Host "[3/3] Accesos directos omitidos (--NoShortcut)." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  =======================================" -ForegroundColor Green
Write-Host "  [OK] SysUtils listo para usarse.       " -ForegroundColor Green
Write-Host "  =======================================" -ForegroundColor Green
Write-Host ""
