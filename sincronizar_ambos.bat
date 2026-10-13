@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul
title Sincronizador Dual WolfDesk & FarmaDesk

echo ==========================================================
echo       🔄 Sincronizador de Repositorios Dual
echo   WolfDesk (WOLFDESK) ^<---^> FarmaDesk (FARMADESK)
echo ==========================================================
echo.

:: 1. Asegurar que estamos en main
echo [*] Cambiando a rama 'main' (WolfDesk)...
git checkout main
if errorlevel 1 (
    echo [!] Error: No se pudo cambiar a la rama 'main'.
    pause
    exit /b 1
)

:: 2. En main, garantizar que README.md sea WolfDesk
if exist README_WOLFDESK.md (
    copy /Y README_WOLFDESK.md README.md >nul
    git add README.md
    git diff-index --quiet HEAD || git commit -m "docs: sync WolfDesk README"
)

:: 3. Empujar a WOLFDESK
echo [*] Enviando actualizaciones a WOLFDESK (origin/main)...
git push origin main
if errorlevel 1 (
    echo [!] Advertencia al enviar a origin/main.
)

:: 4. Cambiar a rama farmadesk
echo.
echo [*] Cambiando a rama 'farmadesk' (FarmaDesk)...
git checkout farmadesk
if errorlevel 1 (
    echo [!] Error: No se pudo cambiar a la rama 'farmadesk'.
    pause
    exit /b 1
)

:: 5. Traer las mejoras de código desde main a farmadesk
echo [*] Integrando cambios del núcleo compartido...
git merge main -m "chore(sync): merge latest core updates into farmadesk"

:: 6. En farmadesk, garantizar que README.md sea FarmaDesk
if exist README_FARMADESK.md (
    copy /Y README_FARMADESK.md README.md >nul
    git add README.md
    git diff-index --quiet HEAD || git commit -m "docs(farmadesk): configure official FarmaDesk Pro branding as repository README"
)

:: 7. Empujar a FARMADESK
echo [*] Enviando actualizaciones a FARMADESK (farmadesk/main)...
git push farmadesk farmadesk:main
if errorlevel 1 (
    echo [!] Advertencia al enviar a farmadesk/main.
)

:: 8. Regresar a la rama main
git checkout main

echo.
echo ==========================================================
echo  ✅ ¡Sincronización completada exitosamente en ambos repos!
echo  - WOLFDESK:  https://github.com/jesusbohada/WOLFDESK
echo  - FARMADESK: https://github.com/jesusbohada/FARMADESK
echo ==========================================================
echo.
pause
