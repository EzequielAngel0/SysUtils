<#
.SYNOPSIS
Manager interactivo para compilar, instalar o desinstalar SysUtils.
#>

$ErrorActionPreference = "Stop"

$AppName       = "SysUtils"
$AppExeName    = "SysUtils.exe"
$AppDataFolder = Join-Path $env:LOCALAPPDATA "SysUtils"
$AppExeDest    = Join-Path $AppDataFolder $AppExeName
# El .ps1 está dentro de phantom_native/, así que $PSScriptRoot ya ES el directorio del proyecto
$NativeDir     = $PSScriptRoot
$DesktopPath   = [Environment]::GetFolderPath("Desktop")
$StartMenuPath = [Environment]::GetFolderPath("Programs")

function Show-Menu {
    Clear-Host
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "              SYSUTILS                   " -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "[1] Compilar y Probar (Modo Local)"
    Write-Host "[2] Instalar SysUtils"
    Write-Host "[3] Desinstalar SysUtils"
    Write-Host "[4] Salir"
    Write-Host ""
}

function Invoke-RunLocal {
    Write-Host "`n[*] Compilando y ejecutando SysUtils..." -ForegroundColor Yellow
    Push-Location $NativeDir
    $env:RUSTFLAGS = "-C target-cpu=native"
    cargo run --release
    Pop-Location
    Write-Host "`nPresiona ENTER para continuar..."
    Read-Host
}

function Invoke-Install {
    if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
        Write-Host "Error: No se encontro 'cargo'. Instala Rust primero." -ForegroundColor Red
        Read-Host "Presiona ENTER para continuar..."
        return
    }

    Write-Host "`n[*] Deteniendo SysUtils si esta abierto..." -ForegroundColor Yellow
    Stop-Process -Name "sysutils_native" -Force -ErrorAction SilentlyContinue
    Stop-Process -Name "SysUtils" -Force -ErrorAction SilentlyContinue

    Write-Host "[*] Compilando con optimizaciones maximas..." -ForegroundColor Yellow
    Push-Location $NativeDir
    $env:RUSTFLAGS = "-C target-cpu=native"
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error en la compilacion." -ForegroundColor Red
        Pop-Location
        Read-Host "Presiona ENTER para continuar..."
        return
    }
    Pop-Location

    Write-Host "[*] Instalando en AppData..." -ForegroundColor Yellow
    if (-not (Test-Path $AppDataFolder)) {
        New-Item -ItemType Directory -Path $AppDataFolder | Out-Null
    }

    # El binario compilado se llama sysutils_native.exe (nombre del crate)
    $CompiledExe = Join-Path $NativeDir "target\release\sysutils_native.exe"
    Copy-Item -Path $CompiledExe -Destination $AppExeDest -Force

    # Copiar icono
    $SourceIcon = Join-Path $NativeDir "assets\icon.ico"
    $DestIcon   = Join-Path $AppDataFolder "icon.ico"
    if (Test-Path $SourceIcon) {
        Copy-Item -Path $SourceIcon -Destination $DestIcon -Force
    }

    Write-Host "[*] Generando accesos directos..." -ForegroundColor Yellow
    $WshShell = New-Object -ComObject WScript.Shell

    $ShortcutDesktop = $WshShell.CreateShortcut((Join-Path $DesktopPath "$AppName.lnk"))
    $ShortcutDesktop.TargetPath       = $AppExeDest
    $ShortcutDesktop.WorkingDirectory = $AppDataFolder
    $ShortcutDesktop.Description      = $AppName
    if (Test-Path $DestIcon) { $ShortcutDesktop.IconLocation = $DestIcon }
    $ShortcutDesktop.Save()

    $ShortcutStart = $WshShell.CreateShortcut((Join-Path $StartMenuPath "$AppName.lnk"))
    $ShortcutStart.TargetPath       = $AppExeDest
    $ShortcutStart.WorkingDirectory = $AppDataFolder
    $ShortcutStart.Description      = $AppName
    if (Test-Path $DestIcon) { $ShortcutStart.IconLocation = $DestIcon }
    $ShortcutStart.Save()

    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Green
    Write-Host " SysUtils instalado correctamente        " -ForegroundColor Green
    Write-Host " Acceso directo creado en el Escritorio  " -ForegroundColor Green
    Write-Host "=========================================" -ForegroundColor Green
    Read-Host "Presiona ENTER para volver al menu..."
}

function Invoke-Uninstall {
    Write-Host "`n[*] Deteniendo SysUtils..." -ForegroundColor Yellow
    Stop-Process -Name "sysutils_native" -Force -ErrorAction SilentlyContinue
    Stop-Process -Name "SysUtils" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1

    Write-Host "[*] Eliminando archivos ($AppDataFolder)..." -ForegroundColor Yellow
    if (Test-Path $AppDataFolder) {
        Remove-Item -Path $AppDataFolder -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[*] Eliminando accesos directos..." -ForegroundColor Yellow
    $deskLnk  = Join-Path $DesktopPath "$AppName.lnk"
    $startLnk = Join-Path $StartMenuPath "$AppName.lnk"
    if (Test-Path $deskLnk)  { Remove-Item -Path $deskLnk  -Force -ErrorAction SilentlyContinue }
    if (Test-Path $startLnk) { Remove-Item -Path $startLnk -Force -ErrorAction SilentlyContinue }

    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Green
    Write-Host " SysUtils desinstalado.                  " -ForegroundColor Green
    Write-Host "=========================================" -ForegroundColor Green
    Read-Host "Presiona ENTER para volver al menu..."
}

while ($true) {
    Show-Menu
    $choice = Read-Host "Elige una opcion"

    switch ($choice) {
        "1" { Invoke-RunLocal }
        "2" { Invoke-Install }
        "3" { Invoke-Uninstall }
        "4" { exit }
        default {
            Write-Host "Opcion invalida." -ForegroundColor Red
            Start-Sleep -Seconds 1
        }
    }
}
