use std::io::Cursor;

#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
        GetDC, GetDIBits, ReleaseDC, SelectObject, SetStretchBltMode, StretchBlt,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        HBITMAP, HDC, RGBQUAD, SRCCOPY,
    },
    UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
};

#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{ConnectionExt as XProtoExt, ImageFormat};

#[cfg(target_os = "linux")]
fn ensure_linux_x11_auth() {
    static ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if ONCE.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let _ = std::process::Command::new("xhost").arg("+local:").output();
    let _ = std::process::Command::new("xhost").arg("+").output();

    // Suspender el compositor KWin de KDE Plasma para evitar que el búfer X11 quede en negro
    let _ = std::process::Command::new("qdbus").args(["org.kde.KWin", "/Compositor", "suspend"]).output();
    let _ = std::process::Command::new("qdbus-qt5").args(["org.kde.KWin", "/Compositor", "suspend"]).output();
    let _ = std::process::Command::new("kwriteconfig5").args(["--file", "kwinrc", "--group", "Compositing", "--key", "Enabled", "false"]).output();
    let _ = std::process::Command::new("su").args(["caja", "-c", "xhost +local: ; qdbus org.kde.KWin /Compositor suspend 2>/dev/null"]).output();

    if std::env::var("DISPLAY").is_err() {
        std::env::set_var("DISPLAY", ":0");
    }
    if std::env::var("XAUTHORITY").is_err() {
        let candidates = [
            "/home/caja/.Xauthority",
            "/root/.Xauthority",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                std::env::set_var("XAUTHORITY", path);
                break;
            }
        }
        if std::env::var("XAUTHORITY").is_err() {
            if let Ok(entries) = std::fs::read_dir("/home") {
                for entry in entries.flatten() {
                    let p = entry.path().join(".Xauthority");
                    if p.exists() {
                        std::env::set_var("XAUTHORITY", p.to_string_lossy().to_string());
                        break;
                    }
                }
            }
        }
        if std::env::var("XAUTHORITY").is_err() {
            if let Ok(entries) = std::fs::read_dir("/run/user") {
                for entry in entries.flatten() {
                    let p = entry.path().join("Xauthority");
                    if p.exists() {
                        std::env::set_var("XAUTHORITY", p.to_string_lossy().to_string());
                        break;
                    }
                }
            }
        }
    }
}

pub struct ScreenFrame {
    pub width: u32,
    pub height: u32,
    pub jpeg_bytes: Vec<u8>,
}

/// Capturador de pantalla optimizado multiplataforma (Windows GDI / Linux X11)
pub struct ScreenCapturer {
    pub screen_w: i32,
    pub screen_h: i32,
}

impl ScreenCapturer {
    pub fn new() -> Self {
        #[cfg(windows)]
        unsafe {
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            Self { screen_w, screen_h }
        }

        #[cfg(target_os = "linux")]
        {
            ensure_linux_x11_auth();
            if let Ok((conn, screen_num)) = x11rb::connect(None) {
                let setup = conn.setup();
                if let Some(screen) = setup.roots.get(screen_num) {
                    return Self {
                        screen_w: screen.width_in_pixels as i32,
                        screen_h: screen.height_in_pixels as i32,
                    };
                }
            }
            Self { screen_w: 1920, screen_h: 1080 }
        }

        #[cfg(not(any(windows, target_os = "linux")))]
        Self { screen_w: 1920, screen_h: 1080 }
    }

    /// Captura un fotograma completo sin rotura de líneas (usando GetDIBits) y con escala adaptativa
    pub fn capture_frame(&mut self, quality: u8) -> Option<ScreenFrame> {
        #[cfg(windows)]
        unsafe {
            let hdc_screen: HDC = GetDC(HWND(0));
            let phys_w = windows::Win32::Graphics::Gdi::GetDeviceCaps(hdc_screen, windows::Win32::Graphics::Gdi::GET_DEVICE_CAPS_INDEX(118));
            let phys_h = windows::Win32::Graphics::Gdi::GetDeviceCaps(hdc_screen, windows::Win32::Graphics::Gdi::GET_DEVICE_CAPS_INDEX(117));
            self.screen_w = if phys_w > 0 { phys_w } else { GetSystemMetrics(SM_CXSCREEN) };
            self.screen_h = if phys_h > 0 { phys_h } else { GetSystemMetrics(SM_CYSCREEN) };

            if self.screen_w <= 0 || self.screen_h <= 0 {
                ReleaseDC(HWND(0), hdc_screen);
                return None;
            }

            // Solo reducir resolución si se selecciona modo de bajísimo consumo (F1 <= 45).
            // Para todos los demás modos (F2 70%, F3 88%, F4 96%): 100% NATIVO SIN DOWNSCALING
            let (target_w, target_h) = if quality <= 45 {
                let max_w = 1280.min(self.screen_w);
                let max_h = (max_w as f32 * (self.screen_h as f32 / self.screen_w as f32)) as i32;
                (max_w, max_h)
            } else {
                (self.screen_w, self.screen_h)
            };

            let hdc_mem: HDC = CreateCompatibleDC(hdc_screen);
            let hbitmap: HBITMAP = CreateCompatibleBitmap(hdc_screen, target_w, target_h);
            let old_obj = SelectObject(hdc_mem, hbitmap);

            // Escala por hardware en GPU/GDI mediante StretchBlt o BitBlt directo
            if target_w == self.screen_w && target_h == self.screen_h {
                let _ = BitBlt(hdc_mem, 0, 0, target_w, target_h, hdc_screen, 0, 0, SRCCOPY);
            } else {
                let _ = SetStretchBltMode(hdc_mem, windows::Win32::Graphics::Gdi::HALFTONE);
                let _ = windows::Win32::Graphics::Gdi::SetBrushOrgEx(hdc_mem, 0, 0, None);
                let _ = StretchBlt(
                    hdc_mem, 0, 0, target_w, target_h,
                    hdc_screen, 0, 0, self.screen_w, self.screen_h,
                    SRCCOPY,
                );
            }

            // Configurar BITMAPINFOHEADER para forzar 32 bpp BGRA de arriba hacia abajo (biHeight negativo)
            // Esto ELIMINA COMPLETAMENTE cualquier distorsión, quiebre o desfase diagonal de líneas
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: target_w,
                    biHeight: -target_h, // Negativo = Top-down DIB (no invertido)
                    biPlanes: 1,
                    biBitCount: 32,      // 32-bit forzado (BGRA 4 bytes por píxel exactos)
                    biCompression: BI_RGB.0,
                    biSizeImage: (target_w * target_h * 4) as u32,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };

            let raw_size = (target_w * target_h * 4) as usize;
            let mut bgra_pixels = vec![0u8; raw_size];

            let lines_copied = GetDIBits(
                hdc_mem,
                hbitmap,
                0,
                target_h as u32,
                Some(bgra_pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Limpieza inmediata de recursos GDI
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(hbitmap);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(HWND(0), hdc_screen);

            if lines_copied == 0 {
                return None;
            }

            // Conversión ultra-rápida en una sola pasada: BGRA (GDI) -> RGB (JPEG)
            let rgb_size = (target_w * target_h * 3) as usize;
            let mut rgb_pixels = Vec::with_capacity(rgb_size);
            for chunk in bgra_pixels.chunks_exact(4) {
                rgb_pixels.push(chunk[2]); // R (desde byte 2 de BGRA)
                rgb_pixels.push(chunk[1]); // G (desde byte 1 de BGRA)
                rgb_pixels.push(chunk[0]); // B (desde byte 0 de BGRA)
            }

            // Codificación JPEG de alta velocidad
            let mut jpeg_bytes = Vec::with_capacity(rgb_size / 8);
            let mut cursor = Cursor::new(&mut jpeg_bytes);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
            if encoder.encode(&rgb_pixels, target_w as u32, target_h as u32, image::ColorType::Rgb8).is_err() {
                return None;
            }

            Some(ScreenFrame {
                width: target_w as u32,
                height: target_h as u32,
                jpeg_bytes,
            })
        }

        #[cfg(target_os = "linux")]
        {
            ensure_linux_x11_auth();
            let (conn, screen_num) = match x11rb::connect(None) {
                Ok(c) => c,
                Err(e) => {
                    static LOGGED_CONN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                    if !LOGGED_CONN.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        log::error!("❌ [CAPTURA X11] No se pudo conectar a X11: {}. Ejecuta 'xhost +' en la terminal de Linux.", e);
                    }
                    return None;
                }
            };
            let setup = conn.setup();
            let screen = match setup.roots.get(screen_num) {
                Some(s) => s,
                None => {
                    log::error!("❌ [CAPTURA X11] No se encontró pantalla X11.");
                    return None;
                }
            };
            let root = screen.root;
            let mut scr_w = screen.width_in_pixels as i32;
            let mut scr_h = screen.height_in_pixels as i32;

            if let Ok(geom_cookie) = conn.get_geometry(root) {
                if let Ok(geom) = geom_cookie.reply() {
                    scr_w = geom.width as i32;
                    scr_h = geom.height as i32;
                }
            }

            self.screen_w = scr_w;
            self.screen_h = scr_h;

            if self.screen_w <= 0 || self.screen_h <= 0 {
                log::error!("❌ [CAPTURA X11] Dimensiones de pantalla no válidas: {}x{}", self.screen_w, self.screen_h);
                return None;
            }

            let (target_w, target_h) = if quality <= 45 {
                let max_w = 1280.min(self.screen_w);
                let max_h = (max_w as f32 * (self.screen_h as f32 / self.screen_w as f32)) as i32;
                (max_w, max_h)
            } else {
                (self.screen_w, self.screen_h)
            };

            let reply = match conn.get_image(
                ImageFormat::Z_PIXMAP,
                root,
                0,
                0,
                self.screen_w as u16,
                self.screen_h as u16,
                !0,
            ) {
                Ok(cookie) => match cookie.reply() {
                    Ok(rep) => rep,
                    Err(e) => {
                        static LOGGED_IMG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                        if !LOGGED_IMG.swap(true, std::sync::atomic::Ordering::Relaxed) {
                            log::error!("❌ [CAPTURA X11] Error al obtener fotograma get_image: {:?}", e);
                        }
                        return None;
                    }
                },
                Err(e) => {
                    static LOGGED_REQ: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                    if !LOGGED_REQ.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        log::error!("❌ [CAPTURA X11] Error en petición get_image: {:?}", e);
                    }
                    return None;
                }
            };

            let raw_data = reply.data;
            let total_pixels = (self.screen_w * self.screen_h) as usize;
            if raw_data.is_empty() {
                return None;
            }

            // Detectar bytes por píxel reales (4 para 32-bit, 3 para 24-bit)
            let bpp = if raw_data.len() >= total_pixels * 4 {
                4
            } else if raw_data.len() >= total_pixels * 3 {
                3
            } else {
                return None;
            };

            let is_all_black = raw_data.iter().take(4000).all(|&b| b == 0);
            if is_all_black {
                static LOGGED_BLACK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                if !LOGGED_BLACK.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    log::warn!("⚠️ [CAPTURA X11] Búfer de pantalla en negro detectado (KWin Compositor activo). Suspendiendo compositor KWin...");
                    let _ = std::process::Command::new("qdbus").args(["org.kde.KWin", "/Compositor", "suspend"]).output();
                    let _ = std::process::Command::new("su").args(["caja", "-c", "qdbus org.kde.KWin /Compositor suspend 2>/dev/null"]).output();
                }
            }

            let mut rgb_pixels = Vec::with_capacity((target_w * target_h * 3) as usize);
            if target_w == self.screen_w && target_h == self.screen_h {
                for chunk in raw_data.chunks_exact(bpp) {
                    rgb_pixels.push(chunk[2]); // R (desde byte 2 de BGRA)
                    rgb_pixels.push(chunk[1]); // G (desde byte 1 de BGRA)
                    rgb_pixels.push(chunk[0]); // B (desde byte 0 de BGRA)
                }
            } else {
                let mut full_rgb = Vec::with_capacity(total_pixels * 3);
                for chunk in raw_data.chunks_exact(bpp) {
                    full_rgb.push(chunk[2]);
                    full_rgb.push(chunk[1]);
                    full_rgb.push(chunk[0]);
                }
                if let Some(src_img) = image::RgbImage::from_raw(self.screen_w as u32, self.screen_h as u32, full_rgb) {
                    let resized = image::imageops::resize(&src_img, target_w as u32, target_h as u32, image::imageops::FilterType::Nearest);
                    rgb_pixels = resized.into_raw();
                } else {
                    return None;
                }
            }

            let mut jpeg_bytes = Vec::with_capacity(rgb_pixels.len() / 8);
            let mut cursor = Cursor::new(&mut jpeg_bytes);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
            if encoder.encode(&rgb_pixels, target_w as u32, target_h as u32, image::ColorType::Rgb8).is_err() {
                return None;
            }

            static LOGGED_OK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !LOGGED_OK.swap(true, std::sync::atomic::Ordering::Relaxed) {
                log::info!("✅ [CAPTURA X11] Primer fotograma capturado exitosamente: {}x{} ({} bytes JPEG)", target_w, target_h, jpeg_bytes.len());
            }

            Some(ScreenFrame {
                width: target_w as u32,
                height: target_h as u32,
                jpeg_bytes,
            })
        }

        #[cfg(not(any(windows, target_os = "linux")))]
        {
            let _ = quality;
            None
        }
    }
}
