@echo off
title WolfDesk Pro - Desinstalador Oficial
echo ============================================================
echo   Desinstalando WolfDesk Pro de este equipo...
echo ============================================================
"%~dp0wolfdesk.exe" --uninstall
if %errorlevel% equ 0 (
    echo.
    echo [OK] WolfDesk y su ID permanente han sido eliminados de este equipo.
    echo Si vuelve a instalar la aplicacion en el futuro, se le asignara un nuevo ID.
) else (
    echo.
    echo [ERROR] Ocurrio un problema durante la desinstalacion.
)
pause
