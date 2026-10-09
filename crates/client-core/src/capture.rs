use std::io::Cursor;

#[cfg(windows)]
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
        EnumDisplayMonitors, GetDC, GetDIBits, GetMonitorInfoW, ReleaseDC, SelectObject,
        SetStretchBltMode, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        HBITMAP, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW, RGBQUAD, SRCCOPY,
    },
    UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN,
        SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    },
};

#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{ConnectionExt as XProtoExt, ImageFormat};

#[derive(Clone, Debug, PartialEq)]
pub struct MonitorBounds {
    pub id: u8,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_primary: bool,
}

#[cfg(windows)]
pub fn enumerate_monitors() -> Vec<MonitorBounds> {
    unsafe extern "system" fn enum_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let list = &mut *(lparam.0 as *mut Vec<MonitorBounds>);
        let mut minfo = MONITORINFOEXW::default();
        minfo.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(hmon, &mut minfo as *mut _ as *mut MONITORINFO).as_bool() {
            let rc = minfo.monitorInfo.rcMonitor;
            let is_primary = (minfo.monitorInfo.dwFlags & 1) != 0;
            let id = (list.len() + 1) as u8;
            let w = rc.right - rc.left;
            let h = rc.bottom - rc.top;
            if w > 0 && h > 0 {
                let name = if is_primary {
                    format!("Pantalla {} (Principal)", id)
                } else {
                    format!("Pantalla {}", id)
                };
                list.push(MonitorBounds {
                    id,
                    name,
                    x: rc.left,
                    y: rc.top,
                    width: w,
                    height: h,
                    is_primary,
                });
            }
        }
        BOOL(1)
    }

    let mut monitors = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC(0),
            None,
            Some(enum_proc),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }
    if monitors.is_empty() {
        let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        monitors.push(MonitorBounds {
            id: 1,
            name: "Pantalla 1 (Principal)".to_string(),
            x: 0,
            y: 0,
            width: if w > 0 { w } else { 1920 },
            height: if h > 0 { h } else { 1080 },
            is_primary: true,
        });
    }
    monitors
}

#[cfg(target_os = "linux")]
pub fn enumerate_monitors() -> Vec<MonitorBounds> {
    ensure_linux_x11_auth();
    let mut monitors = Vec::new();
    if let Ok((conn, screen_num)) = x11rb::connect(None) {
        let setup = conn.setup();
        if let Some(screen) = setup.roots.get(screen_num) {
            let mut w = screen.width_in_pixels as i32;
            let mut h = screen.height_in_pixels as i32;
            if let Ok(geom) = conn.get_geometry(screen.root).and_then(|c| c.reply()) {
                w = geom.width as i32;
                h = geom.height as i32;
            }
            monitors.push(MonitorBounds {
                id: 1,
                name: "Pantalla 1 (Principal)".to_string(),
                x: 0,
                y: 0,
                width: w,
                height: h,
                is_primary: true,
            });
            return monitors;
        }
    }
    monitors.push(MonitorBounds {
        id: 1,
        name: "Pantalla 1 (Principal)".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        is_primary: true,
    });
    monitors
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn enumerate_monitors() -> Vec<MonitorBounds> {
    vec![MonitorBounds {
        id: 1,
        name: "Pantalla 1 (Principal)".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        is_primary: true,
    }]
}

#[cfg(target_os = "linux")]
pub fn ensure_linux_x11_auth() {
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

/// Capturador de pantalla optimizado multiplataforma (Windows GDI / Linux X11) con soporte multi-monitor
pub struct ScreenCapturer {
    pub screen_w: i32,
    pub screen_h: i32,
    pub capture_x: i32,
    pub capture_y: i32,
    pub selected_display: u8,
    pub monitors: Vec<MonitorBounds>,
}

impl ScreenCapturer {
    pub fn new() -> Self {
        let mut capturer = Self {
            screen_w: 1920,
            screen_h: 1080,
            capture_x: 0,
            capture_y: 0,
            selected_display: 1, // Por defecto: Pantalla 1
            monitors: Vec::new(),
        };
        capturer.update_bounds();
        capturer
    }

    /// Cambia el monitor activo a capturar (0 = Todas las pantallas combinadas, 1..N = Pantalla específica)
    pub fn set_display(&mut self, display_id: u8) {
        self.selected_display = display_id;
        self.update_bounds();
    }

    /// Actualiza la lista de monitores y calcula las dimensiones y desplazamientos exactos
    pub fn update_bounds(&mut self) {
        self.monitors = enumerate_monitors();
        if self.selected_display == 0 {
            // Modo "Todas las pantallas" (Virtual Screen)
            #[cfg(windows)]
            unsafe {
                let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
                let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
                let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);
                if vw > 0 && vh > 0 {
                    self.capture_x = vx;
                    self.capture_y = vy;
                    self.screen_w = vw;
                    self.screen_h = vh;
                    return;
                }
            }
        } else if let Some(mon) = self.monitors.iter().find(|m| m.id == self.selected_display) {
            self.capture_x = mon.x;
            self.capture_y = mon.y;
            self.screen_w = mon.width;
            self.screen_h = mon.height;
            return;
        }

        if let Some(first) = self.monitors.first() {
            self.capture_x = first.x;
            self.capture_y = first.y;
            self.screen_w = first.width;
            self.screen_h = first.height;
        } else {
            self.capture_x = 0;
            self.capture_y = 0;
            self.screen_w = 1920;
            self.screen_h = 1080;
        }
    }

    /// Captura un fotograma completo sin rotura de líneas (usando GetDIBits) y con escala adaptativa
    pub fn capture_frame(&mut self, quality: u8) -> Option<ScreenFrame> {
        #[cfg(windows)]
        unsafe {
            let hdc_screen: HDC = GetDC(HWND(0));
            let src_x = self.capture_x;
            let src_y = self.capture_y;
            let src_w = self.screen_w;
            let src_h = self.screen_h;

            if src_w <= 0 || src_h <= 0 {
                ReleaseDC(HWND(0), hdc_screen);
                return None;
            }

            // Solo reducir resolución si se selecciona modo de bajísimo consumo (F1 <= 45).
            // Para todos los demás modos (F2 70%, F3 88%, F4 96%): 100% NATIVO SIN DOWNSCALING
            let (target_w, target_h) = if quality <= 45 {
                let max_w = 1280.min(src_w);
                let max_h = (max_w as f32 * (src_h as f32 / src_w as f32)) as i32;
                (max_w, max_h)
            } else {
                (src_w, src_h)
            };

            let hdc_mem: HDC = CreateCompatibleDC(hdc_screen);
            let hbitmap: HBITMAP = CreateCompatibleBitmap(hdc_screen, target_w, target_h);
            let old_obj = SelectObject(hdc_mem, hbitmap);

            // Escala por hardware en GPU/GDI mediante StretchBlt o BitBlt directo con origen multi-monitor
            if target_w == src_w && target_h == src_h {
                let _ = BitBlt(hdc_mem, 0, 0, target_w, target_h, hdc_screen, src_x, src_y, SRCCOPY);
            } else {
                let _ = SetStretchBltMode(hdc_mem, windows::Win32::Graphics::Gdi::HALFTONE);
                let _ = windows::Win32::Graphics::Gdi::SetBrushOrgEx(hdc_mem, 0, 0, None);
                let _ = StretchBlt(
                    hdc_mem, 0, 0, target_w, target_h,
                    hdc_screen, src_x, src_y, src_w, src_h,
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
