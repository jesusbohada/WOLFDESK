@echo off
setlocal
chcp 65001 >nul
title Servidor de Señalización - FarmaDesk & WolfDesk
echo ============================================================
echo   🚀 SERVIDOR DE SEÑALIZACIÓN MULTI-ORGANIZACIÓN
echo       FarmaDesk Pro (Farmanorte) & WolfDesk Pro
echo ============================================================
echo.
echo [*] Iniciando servidor en el puerto 9050...
echo.

if exist "%~dp0server.exe" (
    "%~dp0server.exe"
) else if exist "%~dp0target\release\server.exe" (
    "%~dp0target\release\server.exe"
) else (
    server.exe
)

if errorlevel 1 (
    echo.
    echo [!] El servidor se ha detenido.
    pause
)
