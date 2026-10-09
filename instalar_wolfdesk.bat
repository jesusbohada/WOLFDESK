@echo off
title WolfDesk Pro - Instalador Oficial
echo ============================================================
echo   Instalando WolfDesk Pro en este equipo...
echo ============================================================

set "EXE_PATH=%~dp0wolfdesk.exe"
if not exist "%EXE_PATH%" set "EXE_PATH=%~dp0target\release\wolfdesk.exe"

if exist "%EXE_PATH%" (
    "%EXE_PATH%" --install
    if %errorlevel% equ 0 (
        echo.
        echo [OK] WolfDesk ha sido instalado exitosamente.
        echo - Acceso directo creado en el Escritorio.
        echo - Acceso directo creado en el Menu Inicio.
        echo - ID criptografico permanente registrado en el sistema.
    ) else (
        echo.
        echo [ERROR] Ocurrio un problema durante la instalacion.
    )
) else (
    echo [ERROR] No se encontro el ejecutable wolfdesk.exe
)
pause
