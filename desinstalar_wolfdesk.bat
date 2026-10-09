@echo off
title WolfDesk Pro - Desinstalador Oficial
echo ============================================================
echo   Desinstalando WolfDesk Pro de este equipo...
echo ============================================================

set "EXE_PATH=%~dp0wolfdesk.exe"
if not exist "%EXE_PATH%" set "EXE_PATH=%~dp0target\release\wolfdesk.exe"

if exist "%EXE_PATH%" (
    "%EXE_PATH%" --uninstall
    if %errorlevel% equ 0 (
        echo.
        echo [OK] WolfDesk y su ID permanente han sido eliminados de este equipo.
        echo Si vuelve a instalar la aplicacion en el futuro, se le asignara un nuevo ID.
    ) else (
        echo.
        echo [ERROR] Ocurrio un problema durante la desinstalacion.
    )
) else (
    echo [ERROR] No se encontro el ejecutable wolfdesk.exe
)
pause
