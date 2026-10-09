# WolfDesk Pro - Script de Actualizacion para Windows
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "        [WolfDesk] - Actualizador (Windows)         " -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan

$RepoDir = Get-Location
if (Test-Path ".git") {
    Write-Host "[*] Repositorio detectado en: $RepoDir" -ForegroundColor Yellow
    Write-Host "[*] Descargando ultimos cambios desde GitHub..." -ForegroundColor Cyan
    git fetch origin main
    git checkout -f main
    git reset --hard origin/main

    # Forzar a Cargo a recompilar con el nuevo commit hash
    if (Test-Path "crates\client-core\build.rs") {
        (Get-Item "crates\client-core\build.rs").LastWriteTime = Get-Date
    }
    $GitHash = git rev-parse --short=7 HEAD
    $env:WOLFDESK_BUILD_GIT_HASH = $GitHash

    # Si wolfdesk esta corriendo, lo cerramos limpiamente
    $RunningProc = Get-Process -Name wolfdesk -ErrorAction SilentlyContinue
    if ($RunningProc) {
        Write-Host "[*] Cerrando instancia activa de WolfDesk en ejecucion..." -ForegroundColor Yellow
        Stop-Process -Name wolfdesk -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
    }

    # Si el archivo wolfdesk.exe sigue en uso, renombramos a .old para permitir la compilacion limpia
    $TargetExe = "target\release\wolfdesk.exe"
    if (Test-Path $TargetExe) {
        $Ts = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
        $OldExe = "target\release\wolfdesk.exe.old_${PID}_${Ts}"
        Rename-Item -Path $TargetExe -NewName "wolfdesk.exe.old_${PID}_${Ts}" -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[*] Compilando binarios con Cargo (Release)..." -ForegroundColor Cyan
    cargo build --release --bin wolfdesk

    if (Test-Path "target\release\wolfdesk.exe") {
        Write-Host "[*] Desplegando wolfdesk.exe en la raiz del proyecto..." -ForegroundColor Cyan
        Copy-Item -Path "target\release\wolfdesk.exe" -Destination "wolfdesk.exe" -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[OK] WolfDesk actualizado exitosamente." -ForegroundColor Green
} else {
    Write-Host "[ERROR] No se detecto un repositorio git en el directorio actual." -ForegroundColor Red
}
