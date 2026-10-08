@echo off
title Instalar Certificado de Confianza WolfDesk
echo Instalando certificado en Autoridades de certificacion raiz de confianza...
certutil -addstore -f "Root" "%~dp0WolfDesk_Certificate.cer"
if %errorlevel% equ 0 (
    echo.
    echo Certificado instalado exitosamente. Windows ahora reconocera a WolfDesk Software como un editor confiable.
) else (
    echo.
    echo Por favor ejecuta este archivo como Administrador haciendo clic derecho -> "Ejecutar como administrador".
)
pause
