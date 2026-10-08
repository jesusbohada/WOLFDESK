@echo off
title WolfDesk - Desbloquear y Ejecutar
powershell -NoProfile -ExecutionPolicy Bypass -Command "Unblock-File -Path '%~dp0wolfdesk.exe'"
start "" "%~dp0wolfdesk.exe"
