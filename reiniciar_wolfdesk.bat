@echo off
title Reiniciando WolfDesk...
echo ==============================================
echo        REINICIO RAPIDO DE WOLFDESK
echo ==============================================
echo 1. Cerrando instancias anteriores de WolfDesk...
taskkill /F /IM wolfdesk.exe /T >nul 2>&1
timeout /t 1 /nobreak >nul

echo 2. Iniciando WolfDesk de forma limpia...
if exist "%~dp0wolfdesk.exe" (
    start "" "%~dp0wolfdesk.exe"
    echo [OK] WolfDesk iniciado correctamente.
) else if exist "%~dp0target\release\wolfdesk.exe" (
    start "" "%~dp0target\release\wolfdesk.exe"
    echo [OK] WolfDesk iniciado correctamente.
) else (
    start "" wolfdesk.exe
    echo [OK] Comando de inicio enviado.
)
timeout /t 2 /nobreak >nul
