@echo off
color 0b
title SysUtils

:: Ejecutar el script PowerShell Manager.ps1 situado en el mismo nivel
powershell -ExecutionPolicy Bypass -NoProfile -File "%~dp0Manager.ps1"
