param(
    [switch]$SkipBuild,
    [switch]$NoShortcut
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$NativeDir = $ProjectRoot
$TargetDir = Join-Path $NativeDir "target\release"
$ExeName = "phantom_native.exe"
$AppName = "Phantom"
$IconSource = Join-Path $NativeDir "assets\icon.png"

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Magenta
Write-Host "       PHANTOM - Build & Deploy           " -ForegroundColor Magenta
Write-Host "  ========================================" -ForegroundColor Magenta
Write-Host ""

# -- Step 1: Build ----------------------------------------------------------------
if (-not $SkipBuild) {
    Write-Host "[1/3] Preparando entorno y compilando en modo Release..." -ForegroundColor Cyan

    # Refresh PATH to include cargo
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "  [!] Rust y Cargo no detectados. Instalando automaticamente..." -ForegroundColor Yellow
        $RustupPath = Join-Path $env:TEMP "rustup-init.exe"
        try {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile $RustupPath
            Write-Host "  Descarga completada. Ejecutando instalacion silenciosa (esto puede tardar unos minutos)..." -ForegroundColor Cyan
            
            # Formato desatendido para rustup
            $process = Start-Process -FilePath $RustupPath -ArgumentList "-y", "--quiet" -Wait -PassThru
            
            if ($process.ExitCode -ne 0) {
                Write-Host "  [X] Error al instalar Rust. Instalalo manualmente desde https://rustup.rs/" -ForegroundColor Red
                exit 1
            }
            
            Write-Host "  [OK] Rust instalado correctamente." -ForegroundColor Green
            # Forzar actualizacion del PATH en la sesion actual
            $CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
            $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + $CargoBin
        } catch {
            Write-Host "  [X] No se pudo descargar el instalador. Instala Rust manualmente: https://rustup.rs/" -ForegroundColor Red
            exit 1
        }
    }

    Push-Location $NativeDir
    try {
        cargo build --release 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  [X] Error de compilacion." -ForegroundColor Red
            Pop-Location
            exit 1
        }
        Write-Host "  [OK] Compilacion exitosa." -ForegroundColor Green
    }
    finally {
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
    Write-Host "[3/3] Creando acceso directo..." -ForegroundColor Cyan

    $DesktopPath = [Environment]::GetFolderPath("Desktop")
    $ShortcutPath = Join-Path $DesktopPath "$AppName.lnk"

    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $ExePath
    $Shortcut.WorkingDirectory = $TargetDir
    $Shortcut.Description = "Phantom - Hardware Automation Suite"
    $Shortcut.WindowStyle = 1
    $Shortcut.Save()

    Write-Host "  [OK] Acceso directo creado en: $ShortcutPath" -ForegroundColor Green

    # Also create in Start Menu for taskbar pinning
    $StartMenuPath = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
    $StartShortcutPath = Join-Path $StartMenuPath "$AppName.lnk"

    $StartShortcut = $WshShell.CreateShortcut($StartShortcutPath)
    $StartShortcut.TargetPath = $ExePath
    $StartShortcut.WorkingDirectory = $TargetDir
    $StartShortcut.Description = "Phantom - Hardware Automation Suite"
    $StartShortcut.WindowStyle = 1
    $StartShortcut.Save()

    Write-Host "  [OK] Acceso en Menu Inicio: $StartShortcutPath" -ForegroundColor Green
    Write-Host ""
    Write-Host "  TIP: Para anclar a la barra de tareas, busca 'Phantom'" -ForegroundColor Yellow
    Write-Host "       en el Menu Inicio y haz clic derecho -> 'Anclar'." -ForegroundColor Yellow
} else {
    Write-Host "[3/3] Creacion de accesos directos omitida (--NoShortcut)." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  =======================================" -ForegroundColor Green
Write-Host "  [OK] Phantom esta listo para usarse." -ForegroundColor Green
Write-Host "  =======================================" -ForegroundColor Green
Write-Host ""
