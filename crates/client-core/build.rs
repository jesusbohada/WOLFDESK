use std::fs;
use std::path::Path;

fn main() {
    let assets_dir = Path::new("assets");
    let _ = fs::create_dir_all(assets_dir);

    let png_path = assets_dir.join("wolfdesk.png");
    let ico_path = assets_dir.join("wolfdesk.ico");

    let src_logo = Path::new(r"C:\Users\sun_d\.gemini\antigravity\brain\8d07daa8-06e3-4769-93a2-ec35f188e6ed\wolfdesk_logo_1791320491781.jpg");

    if src_logo.exists() && (!png_path.exists() || !ico_path.exists()) {
        if let Ok(img) = image::open(src_logo) {
            let resized = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
            let _ = resized.save(&png_path);

            if let Ok(png_data) = fs::read(&png_path) {
                let mut ico = Vec::new();
                // 1. Header de archivo .ICO (6 bytes)
                ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]);

                // 2. Entrada de directorio (16 bytes para PNG de 256x256)
                ico.push(0); // Ancho: 0 representa 256px
                ico.push(0); // Alto: 0 representa 256px
                ico.push(0); // Colores
                ico.push(0); // Reservado
                ico.extend_from_slice(&[1, 0]); // Planos de color = 1
                ico.extend_from_slice(&[32, 0]); // Profundidad de 32 bits
                let size = png_data.len() as u32;
                ico.extend_from_slice(&size.to_le_bytes()); // Tamaño de datos PNG
                let offset = 22u32; // Offset inicial de datos (6 + 16)
                ico.extend_from_slice(&offset.to_le_bytes());

                // 3. Datos comprimidos del PNG
                ico.extend_from_slice(&png_data);
                let _ = fs::write(&ico_path, ico);
            }
        }
    }

    #[cfg(windows)]
    {
        if ico_path.exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/wolfdesk.ico");
            res.set("ProductName", "WolfDesk Pro");
            res.set("FileDescription", "WolfDesk - Software de Escritorio Remoto Seguro");
            res.set("CompanyName", "WolfDesk Software");
            res.set("LegalCopyright", "Copyright (C) 2026 WolfDesk Software. Todos los derechos reservados.");
            res.set("OriginalFilename", "wolfdesk.exe");
            res.set("InternalName", "wolfdesk");
            res.set("ProductVersion", "1.2.0.0");
            res.set("FileVersion", "1.2.0.0");
            res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <assemblyIdentity version="1.2.0.0" processorArchitecture="*" name="WolfDesk.Software.WolfDesk" type="win32"/>
  <description>WolfDesk Remote Desktop</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <asmv3:application>
    <asmv3:windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">True/PM</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </asmv3:windowsSettings>
  </asmv3:application>
</assembly>
"#);
            let _ = res.compile();
        }
    }

    // Exportar el hash corto del commit actual de Git a tiempo de compilación
    let git_hash = std::env::var("WOLFDESK_BUILD_GIT_HASH")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "--short=7", "HEAD"])
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "10eb531".to_string());
    println!("cargo:rustc-env=WOLFDESK_BUILD_GIT_HASH={}", git_hash);

    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let _ = fs::write(Path::new(&out_dir).join("build_git_hash.txt"), &git_hash);
    }

    // Instrucciones a Cargo para re-ejecutar build.rs si cambian referencias de Git
    println!("cargo:rerun-if-env-changed=WOLFDESK_BUILD_GIT_HASH");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");
    println!("cargo:rerun-if-changed=../../.git/refs/heads/main");
    println!("cargo:rerun-if-changed=../../.git/packed-refs");
}
