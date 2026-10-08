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

pub struct ScreenFrame {
    pub width: u32,
    pub height: u32,
    pub jpeg_bytes: Vec<u8>,
}

/// Capturador de pantalla optimizado para Windows con GetDIBits y aceleración GDI
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
        #[cfg(not(windows))]
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

        #[cfg(not(windows))]
        None
    }
}
