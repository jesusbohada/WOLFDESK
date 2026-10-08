@echo off
title WolfDesk Pro - Instalador Oficial
echo ============================================================
echo   Instalando WolfDesk Pro en este equipo...
echo ============================================================
"%~dp0wolfdesk.exe" --install
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
pause
