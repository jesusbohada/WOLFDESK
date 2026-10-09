@echo off
title WolfDesk Pro - Actualizador Oficial
echo ============================================================
echo   🐺 Actualizando WolfDesk a la ultima version...
echo ============================================================
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\update.ps1"
if %errorlevel% equ 0 (
    echo.
    echo [OK] Proceso de actualizacion completado con exito.
) else (
    echo.
    echo [ERROR] Ocurrio un problema durante la actualizacion.
)
pause
