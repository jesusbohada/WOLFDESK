# WolfDesk Pro - Script de Actualización para Windows
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "        🐺 WolfDesk - Actualizador (Windows)        " -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan

$RepoDir = Get-Location
if (Test-Path ".git") {
    Write-Host "[*] Repositorio detectado en: $RepoDir" -ForegroundColor Yellow
    Write-Host "[*] Descargando últimos cambios desde GitHub..." -ForegroundColor Cyan
    git fetch origin main
    git checkout main
    git pull origin main

    # Si wolfdesk.exe está en ejecución, Windows bloquea sobrescribirlo directamente. Renombramos a .old para permitir la compilación limpia.
    $TargetExe = "target\release\wolfdesk.exe"
    if (Test-Path $TargetExe) {
        $Ts = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
        $OldExe = "target\release\wolfdesk.exe.old_${PID}_${Ts}"
        Rename-Item -Path $TargetExe -NewName "wolfdesk.exe.old_${PID}_${Ts}" -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[*] Compilando binarios con Cargo (Release)..." -ForegroundColor Cyan
    cargo build --release --bin wolfdesk

    Write-Host "[✓] ¡WolfDesk actualizado exitosamente!" -ForegroundColor Green
} else {
    Write-Host "[!] No se detectó un repositorio git en el directorio actual." -ForegroundColor Red
}
