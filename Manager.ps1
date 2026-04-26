<#
.SYNOPSIS
Manager interactivo para compilar, instalar o desinstalar Phantom.
#>

$ErrorActionPreference = "Stop"

$AppExeName = "Phantom.exe"
$AppDataFolder = Join-Path $env:LOCALAPPDATA "Phantom"
$PhantomExeDest = Join-Path $AppDataFolder $AppExeName
$NativeDir = Join-Path $PSScriptRoot "phantom_native"
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$StartMenuPath = [Environment]::GetFolderPath("Programs")

function Show-Menu {
    Clear-Host
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "            PHANTOM MANAGER              " -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "`n[1] Compilar y Probar (Modo Local)"
    Write-Host "[2] Instalar Phantom (App Completa)"
    Write-Host "[3] Desinstalar Phantom"
    Write-Host "[4] Salir`n"
}

function Invoke-RunLocal {
    Write-Host "`n[*] Compilando y ejecutando Phantom..." -ForegroundColor Yellow
    Push-Location $NativeDir
    $env:RUSTFLAGS="-C target-cpu=native"
    cargo run --release
    Pop-Location
    Write-Host "`nPresiona ENTER para continuar..."
    Read-Host
}

function Invoke-Install {
    if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
        Write-Host "Error: No se encontró 'cargo'. Instala Rust primero." -ForegroundColor Red
        Read-Host "Presiona ENTER para continuar..."
        return
    }

    Write-Host "`n[*] Deteniendo Phatom si está abierto..."
    Stop-Process -Name "phantom_native" -Force -ErrorAction SilentlyContinue
    Stop-Process -Name "Phantom" -Force -ErrorAction SilentlyContinue

    Write-Host "[*] Compilando Phantom Native con optimizaciones máximas..." -ForegroundColor Yellow
    Push-Location $NativeDir
    $env:RUSTFLAGS="-C target-cpu=native"
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error en la compilación." -ForegroundColor Red
        Pop-Location
        Read-Host "Presiona ENTER para continuar..."
        return
    }
    Pop-Location

    Write-Host "[*] Instalando motor base en AppData..." -ForegroundColor Yellow
    if (-not (Test-Path $AppDataFolder)) {
        New-Item -ItemType Directory -Path $AppDataFolder | Out-Null
    }

    $CompiledExe = Join-Path $NativeDir "target\release\phantom_native.exe"
    Copy-Item -Path $CompiledExe -Destination $PhantomExeDest -Force
    
    # Copiar Icono
    $SourceIcon = Join-Path $NativeDir "assets\icon.ico"
    $DestIcon = Join-Path $AppDataFolder "icon.ico"
    if (Test-Path $SourceIcon) {
        Copy-Item -Path $SourceIcon -Destination $DestIcon -Force
    }

    Write-Host "[*] Generando accesos directos de Windows..." -ForegroundColor Yellow
    $WshShell = New-Object -comObject WScript.Shell

    $ShortcutDesktop = $WshShell.CreateShortcut((Join-Path $DesktopPath "Phantom.lnk"))
    $ShortcutDesktop.TargetPath = $PhantomExeDest
    $ShortcutDesktop.WorkingDirectory = $AppDataFolder
    $ShortcutDesktop.Description = "Phantom Automation"
    if (Test-Path $DestIcon) { $ShortcutDesktop.IconLocation = $DestIcon }
    $ShortcutDesktop.Save()

    $ShortcutStart = $WshShell.CreateShortcut((Join-Path $StartMenuPath "Phantom.lnk"))
    $ShortcutStart.TargetPath = $PhantomExeDest
    $ShortcutStart.WorkingDirectory = $AppDataFolder
    $ShortcutStart.Description = "Phantom Automation"
    if (Test-Path $DestIcon) { $ShortcutStart.IconLocation = $DestIcon }
    $ShortcutStart.Save()

    Write-Host "`n=========================================" -ForegroundColor Green
    Write-Host " ¡Phantom se instaló correctamente!      " -ForegroundColor Green
    Write-Host "=========================================" -ForegroundColor Green
    Read-Host "Presiona ENTER para volver al menú..."
}

function Invoke-Uninstall {
    Write-Host "`n[*] Deteniendo Phantom si está activo..." -ForegroundColor Yellow
    Stop-Process -Name "phantom_native" -Force -ErrorAction SilentlyContinue
    Stop-Process -Name "Phantom" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1

    Write-Host "[*] Limpiando archivos del sistema ($AppDataFolder)..." -ForegroundColor Yellow
    if (Test-Path $AppDataFolder) {
        Remove-Item -Path $AppDataFolder -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[*] Eliminando accesos directos..." -ForegroundColor Yellow
    $deskLnk = Join-Path $DesktopPath "Phantom.lnk"
    if (Test-Path $deskLnk) { Remove-Item -Path $deskLnk -Force -ErrorAction SilentlyContinue }

    $startLnk = Join-Path $StartMenuPath "Phantom.lnk"
    if (Test-Path $startLnk) { Remove-Item -Path $startLnk -Force -ErrorAction SilentlyContinue }

    Write-Host "`n=========================================" -ForegroundColor Green
    Write-Host " Phantom ha sido desinstalado. " -ForegroundColor Green
    Write-Host "=========================================" -ForegroundColor Green
    Read-Host "Presiona ENTER para volver al menú..."
}

while ($true) {
    Show-Menu
    $choice = Read-Host "Elige una opción"
    
    switch ($choice) {
        "1" { Invoke-RunLocal }
        "2" { Invoke-Install }
        "3" { Invoke-Uninstall }
        "4" { exit }
        default { Write-Host "Opción inválida." -ForegroundColor Red; Start-Sleep -Seconds 1 }
    }
}
