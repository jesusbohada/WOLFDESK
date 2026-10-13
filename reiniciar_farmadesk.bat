@echo off
title Reiniciando FarmaDesk...
echo ==============================================
echo        REINICIO RAPIDO DE FARMADESK
echo ==============================================
echo 1. Cerrando instancias anteriores de FarmaDesk...
taskkill /F /IM farmadesk.exe /T >nul 2>&1
timeout /t 1 /nobreak >nul

echo 2. Iniciando FarmaDesk de forma limpia...
if exist "%~dp0farmadesk.exe" (
    start "" "%~dp0farmadesk.exe"
    echo [OK] FarmaDesk iniciado correctamente.
) else if exist "%~dp0target\release\farmadesk.exe" (
    start "" "%~dp0target\release\farmadesk.exe"
    echo [OK] FarmaDesk iniciado correctamente.
) else (
    start "" farmadesk.exe
    echo [OK] Comando de inicio enviado.
)
timeout /t 2 /nobreak >nul
