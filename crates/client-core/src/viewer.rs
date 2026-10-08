use crate::recorder::SessionRecorder;
use minifb::{Key, KeyRepeat, MouseButton, ScaleMode, Window, WindowOptions};
use proto::{ControlEvent, MouseButtonType};
use std::sync::mpsc;
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::ScreenToClient,
    UI::Input::KeyboardAndMouse::GetAsyncKeyState,
    UI::WindowsAndMessaging::{
        CallNextHookEx, GetClientRect, GetCursorPos, GetForegroundWindow, SetWindowsHookExW,
        UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

const TOOLBAR_HEIGHT: usize = 34;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScaleAdaptation {
    AspectRatioFit, // Proporción nativa perfecta con barras oscuras (Recomendado)
    Stretch,        // Estirar imagen para llenar 100% la ventana
    Original100,    // Tamaño 1:1 original centrado píxel por píxel
}

impl ScaleAdaptation {
    pub fn label(&self) -> &'static str {
        match self {
            ScaleAdaptation::AspectRatioFit => "Ajustar (16:9)",
            ScaleAdaptation::Stretch => "Estirar (100%)",
            ScaleAdaptation::Original100 => "Original (1:1)",
        }
    }
}

/// Mapea caracteres ASCII a mapas de bits de 8x8 píxeles
fn get_glyph(c: char) -> [u8; 8] {
    match c {
        'A' | 'a' => [0x18, 0x24, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
        'B' | 'b' => [0x7C, 0x22, 0x22, 0x3C, 0x22, 0x22, 0x7C, 0x00],
        'C' | 'c' => [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00],
        'D' | 'd' => [0x78, 0x24, 0x22, 0x22, 0x22, 0x24, 0x78, 0x00],
        'E' | 'e' => [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x7E, 0x00],
        'F' | 'f' => [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x40, 0x00],
        'G' | 'g' => [0x3C, 0x42, 0x40, 0x4E, 0x42, 0x42, 0x3C, 0x00],
        'H' | 'h' => [0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
        'I' | 'i' => [0x3E, 0x1C, 0x08, 0x08, 0x08, 0x1C, 0x3E, 0x00],
        'J' | 'j' => [0x02, 0x02, 0x02, 0x02, 0x42, 0x42, 0x3C, 0x00],
        'K' | 'k' => [0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00],
        'L' | 'l' => [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00],
        'M' | 'm' => [0x42, 0x66, 0x5A, 0x42, 0x42, 0x42, 0x42, 0x00],
        'N' | 'n' => [0x42, 0x62, 0x52, 0x4A, 0x46, 0x42, 0x42, 0x00],
        'O' | 'o' => [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'P' | 'p' => [0x7C, 0x42, 0x42, 0x7C, 0x40, 0x40, 0x40, 0x00],
        'Q' | 'q' => [0x3C, 0x42, 0x42, 0x42, 0x4A, 0x44, 0x3A, 0x00],
        'R' | 'r' => [0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x00],
        'S' | 's' => [0x3C, 0x42, 0x40, 0x3C, 0x02, 0x42, 0x3C, 0x00],
        'T' | 't' => [0x7F, 0x49, 0x08, 0x08, 0x08, 0x08, 0x1C, 0x00],
        'U' | 'u' => [0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'V' | 'v' => [0x42, 0x42, 0x42, 0x42, 0x24, 0x24, 0x18, 0x00],
        'W' | 'w' => [0x42, 0x42, 0x42, 0x42, 0x5A, 0x66, 0x42, 0x00],
        'X' | 'x' => [0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x42, 0x00],
        'Y' | 'y' => [0x42, 0x42, 0x24, 0x18, 0x08, 0x08, 0x1C, 0x00],
        'Z' | 'z' => [0x7E, 0x04, 0x08, 0x10, 0x20, 0x40, 0x7E, 0x00],
        '0' => [0x3C, 0x46, 0x4A, 0x52, 0x62, 0x42, 0x3C, 0x00],
        '1' => [0x18, 0x28, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00],
        '2' => [0x3C, 0x42, 0x02, 0x0C, 0x30, 0x40, 0x7E, 0x00],
        '3' => [0x3C, 0x42, 0x02, 0x1C, 0x02, 0x42, 0x3C, 0x00],
        '4' => [0x0C, 0x14, 0x24, 0x44, 0x7E, 0x04, 0x04, 0x00],
        '5' => [0x7E, 0x40, 0x7C, 0x02, 0x02, 0x42, 0x3C, 0x00],
        '6' => [0x3C, 0x40, 0x7C, 0x42, 0x42, 0x42, 0x3C, 0x00],
        '7' => [0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x20, 0x00],
        '8' => [0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x3C, 0x00],
        '9' => [0x3C, 0x42, 0x42, 0x3E, 0x02, 0x02, 0x3C, 0x00],
        ':' => [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00],
        '%' => [0x62, 0x64, 0x08, 0x10, 0x26, 0x46, 0x00, 0x00],
        '[' => [0x1E, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1E, 0x00],
        ']' => [0x3C, 0x04, 0x04, 0x04, 0x04, 0x04, 0x3C, 0x00],
        '(' => [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00],
        ')' => [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '+' => [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00],
        '|' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        '/' => [0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        '>' => [0x40, 0x20, 0x10, 0x08, 0x10, 0x20, 0x40, 0x00],
        '<' => [0x02, 0x04, 0x08, 0x10, 0x08, 0x04, 0x02, 0x00],
        _ => [0x00; 8],
    }
}

fn draw_text(buffer: &mut [u32], buf_w: usize, buf_h: usize, start_x: usize, start_y: usize, text: &str, color: u32) {
    let mut cur_x = start_x;
    for c in text.chars() {
        let glyph = get_glyph(c);
        for row in 0..8 {
            let py = start_y + row;
            if py >= buf_h { break; }
            let byte = glyph[row];
            for col in 0..8 {
                let px = cur_x + col;
                if px >= buf_w { break; }
                if (byte & (0x80 >> col)) != 0 {
                    buffer[py * buf_w + px] = color;
                }
            }
        }
        cur_x += 8;
    }
}

fn draw_button(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    rect: (usize, usize, usize, usize), // (x, y, w, h)
    label: &str,
    is_hovered: bool,
    is_active: bool,
    active_color: u32,
) {
    let (rx, ry, rw, rh) = rect;
    let bg_color = if is_active {
        active_color
    } else if is_hovered {
        0x1E293B
    } else {
        0x0F172A
    };
    let border_color = if is_active || is_hovered {
        0x00E5FF
    } else {
        0x334155
    };

    for y in ry..(ry + rh) {
        if y >= buf_h { break; }
        for x in rx..(rx + rw) {
            if x >= buf_w { break; }
            if y == ry || y == ry + rh - 1 || x == rx || x == rx + rw - 1 {
                buffer[y * buf_w + x] = border_color;
            } else {
                buffer[y * buf_w + x] = bg_color;
            }
        }
    }

    let text_len = label.len() * 8;
    let text_x = if rw > text_len { rx + (rw - text_len) / 2 } else { rx + 4 };
    let text_y = ry + (rh.saturating_sub(8)) / 2;
    draw_text(buffer, buf_w, buf_h, text_x, text_y, label, 0xFFFFFF);
}

/// Mapea las teclas de minifb a Virtual-Key codes de Windows (VK_*)
fn minifb_key_to_vk(key: Key) -> Option<u16> {
    let vk = match key {
        Key::A => 0x41, Key::B => 0x42, Key::C => 0x43, Key::D => 0x44,
        Key::E => 0x45, Key::F => 0x46, Key::G => 0x47, Key::H => 0x48,
        Key::I => 0x49, Key::J => 0x4A, Key::K => 0x4B, Key::L => 0x4C,
        Key::M => 0x4D, Key::N => 0x4E, Key::O => 0x4F, Key::P => 0x50,
        Key::Q => 0x51, Key::R => 0x52, Key::S => 0x53, Key::T => 0x54,
        Key::U => 0x55, Key::V => 0x56, Key::W => 0x57, Key::X => 0x58,
        Key::Y => 0x59, Key::Z => 0x5A,

        Key::Key0 => 0x30, Key::Key1 => 0x31, Key::Key2 => 0x32, Key::Key3 => 0x33,
        Key::Key4 => 0x34, Key::Key5 => 0x35, Key::Key6 => 0x36, Key::Key7 => 0x37,
        Key::Key8 => 0x38, Key::Key9 => 0x39,

        Key::LeftSuper => 0x5B,
        Key::RightSuper => 0x5C,
        Key::Menu => 0x5D,
        Key::LeftShift | Key::RightShift => 0x10,
        Key::LeftCtrl | Key::RightCtrl => 0x11,
        Key::LeftAlt | Key::RightAlt => 0x12,

        Key::Space => 0x20,
        Key::Enter => 0x0D,
        Key::Backspace => 0x08,
        Key::Tab => 0x09,
        Key::Escape => 0x1B,
        Key::Left => 0x25,
        Key::Up => 0x26,
        Key::Right => 0x27,
        Key::Down => 0x28,
        Key::Delete => 0x2E,
        Key::Home => 0x24,
        Key::End => 0x23,
        Key::PageUp => 0x21,
        Key::PageDown => 0x22,
        Key::Insert => 0x2D,

        Key::Minus => 0xBD,
        Key::Equal => 0xBB,
        Key::Comma => 0xBC,
        Key::Period => 0xBE,
        Key::Slash => 0xBF,
        Key::Semicolon => 0xBA,
        Key::Apostrophe => 0xDE,
        Key::LeftBracket => 0xDB,
        Key::Backslash => 0xDC,
        Key::RightBracket => 0xDD,
        Key::Backquote => 0xC0,

        Key::NumPad0 => 0x60, Key::NumPad1 => 0x61, Key::NumPad2 => 0x62, Key::NumPad3 => 0x63,
        Key::NumPad4 => 0x64, Key::NumPad5 => 0x65, Key::NumPad6 => 0x66, Key::NumPad7 => 0x67,
        Key::NumPad8 => 0x68, Key::NumPad9 => 0x69,
        Key::NumPadEnter => 0x0D,
        _ => return None,
    };
    Some(vk)
}

#[cfg(windows)]
static HOOK_VIEWER_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
#[cfg(windows)]
static HOOK_TX: std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<ControlEvent>>> = std::sync::Mutex::new(None);

#[cfg(windows)]
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        // Interceptar tecla Windows física (VK_LWIN = 0x5B, VK_RWIN = 0x5C)
        if kbd.vkCode == 0x5B || kbd.vkCode == 0x5C {
            let fg = GetForegroundWindow();
            let target_hwnd = HOOK_VIEWER_HWND.load(std::sync::atomic::Ordering::Relaxed);
            if fg.0 != 0 && fg.0 == target_hwnd {
                let is_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
                let is_up = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;
                if is_down || is_up {
                    if let Ok(guard) = HOOK_TX.lock() {
                        if let Some(tx) = guard.as_ref() {
                            let _ = tx.send(ControlEvent::Keyboard {
                                vk_code: kbd.vkCode as u16,
                                down: is_down,
                            });
                        }
                    }
                }
                // Retornar 1 para suprimir el menú de inicio local en el equipo cliente
                return LRESULT(1);
            }
        }
    }
    CallNextHookEx(HHOOK(0), n_code, wparam, lparam)
}

pub fn start_viewer_window(
    title: &str,
    frame_rx: mpsc::Receiver<Vec<u8>>,
    control_tx: tokio::sync::mpsc::UnboundedSender<ControlEvent>,
) {
    let initial_width = 1600;
    let initial_height = 900;
    let mut buffer: Vec<u32> = vec![0x080C14; initial_width * initial_height];

    let mut window = match Window::new(
        title,
        initial_width,
        initial_height,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            scale_mode: ScaleMode::Stretch, // El buffer siempre coincide con el cliente de la ventana
            ..WindowOptions::default()
        },
    ) {
        Ok(win) => win,
        Err(err) => {
            log::error!("Error al abrir visor: {}", err);
            return;
        }
    };

    window.limit_update_rate(Some(Duration::from_micros(16600))); // 60 FPS
    window.set_key_repeat_delay(0.250); // 250 ms retardo de auto-repetición
    window.set_key_repeat_rate(0.035);  // ~28 repeticiones por segundo (borrado fluido)

    // Configuración del hook de bajo nivel de teclado para capturar la tecla Windows
    #[cfg(windows)]
    let hook: Option<HHOOK> = {
        let hwnd_val = window.get_window_handle() as isize;
        HOOK_VIEWER_HWND.store(hwnd_val, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut guard) = HOOK_TX.lock() {
            *guard = Some(control_tx.clone());
        }
        unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                HINSTANCE(0),
                0,
            ).ok()
        }
    };

    let mut remote_w: usize = 1920;
    let mut remote_h: usize = 1080;
    let mut remote_frame: Vec<u32> = vec![0; remote_w * remote_h];

    let mut last_mouse_pos: Option<(f32, f32)> = None;
    let mut left_was_down = false;
    let mut right_was_down = false;

    let mut recorder = SessionRecorder::new();
    let mut f9_was_pressed = false;
    let mut f5_was_pressed = false;

    // Perfiles dinámicos: por defecto HD Nativa (88%)
    let mut quality_tier = 2u8; // 0=40%, 1=70%, 2=88%, 3=96%
    let mut current_quality_label = "HD Nativa (88%)";
    let mut current_scale_mode = ScaleAdaptation::AspectRatioFit;
    let mut show_toolbar = true;

    println!("\n============================================================");
    println!(" 🐺 [VISOR WOLFDESK HD - RENDIMIENTO TOTAL]");
    println!(" • Barra superior interactiva disponible:");
    println!("   [Calidad], [Escala], [Grabar], [Win Key], [Ocultar], [Salir]");
    println!(" • Atajos de teclado: [F1-F4 Calidad], [F5 Escala], [F9 Grabar]");
    println!(" • Tecla Windows y eliminación continua de teclado habilitadas");
    println!("============================================================");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // 1. Hotkeys de Calidad (F1, F2, F3, F4)
        if window.is_key_down(Key::F1) {
            quality_tier = 0;
            current_quality_label = "Velocidad (40%)";
            let _ = control_tx.send(ControlEvent::SetQuality { quality: 40 });
        } else if window.is_key_down(Key::F2) {
            quality_tier = 1;
            current_quality_label = "Equilibrada (70%)";
            let _ = control_tx.send(ControlEvent::SetQuality { quality: 70 });
        } else if window.is_key_down(Key::F3) {
            quality_tier = 2;
            current_quality_label = "HD Nativa (88%)";
            let _ = control_tx.send(ControlEvent::SetQuality { quality: 88 });
        } else if window.is_key_down(Key::F4) {
            quality_tier = 3;
            current_quality_label = "Ultra HD Cristalina (96%)";
            let _ = control_tx.send(ControlEvent::SetQuality { quality: 96 });
        }

        // 2. Hotkey de Escala (F5)
        let f5_down = window.is_key_down(Key::F5);
        if f5_down && !f5_was_pressed {
            current_scale_mode = match current_scale_mode {
                ScaleAdaptation::AspectRatioFit => ScaleAdaptation::Stretch,
                ScaleAdaptation::Stretch => ScaleAdaptation::Original100,
                ScaleAdaptation::Original100 => ScaleAdaptation::AspectRatioFit,
            };
        }
        f5_was_pressed = f5_down;

        // 3. Hotkey de Grabación (F9)
        let f9_down = window.is_key_down(Key::F9);
        if f9_down && !f9_was_pressed {
            if !recorder.is_recording {
                let _ = recorder.start(title);
            } else {
                let _ = recorder.stop();
            }
        }
        f9_was_pressed = f9_down;

        // 4. Procesar fotogramas recibidos del host
        let mut latest_frame: Option<Vec<u8>> = None;
        while let Ok(jpeg_bytes) = frame_rx.try_recv() {
            recorder.record_frame(&jpeg_bytes);
            latest_frame = Some(jpeg_bytes);
        }

        if let Some(jpeg_bytes) = latest_frame {
            if let Ok(img) = image::load_from_memory(&jpeg_bytes) {
                let rgb = img.to_rgb8();
                let (w, h) = rgb.dimensions();
                let (w_usize, h_usize) = (w as usize, h as usize);

                if w_usize != remote_w || h_usize != remote_h {
                    remote_w = w_usize;
                    remote_h = h_usize;
                    remote_frame.resize(remote_w * remote_h, 0);
                }

                let raw = rgb.as_raw();
                for (i, pixel) in raw.chunks_exact(3).enumerate() {
                    let r = pixel[0] as u32;
                    let g = pixel[1] as u32;
                    let b = pixel[2] as u32;
                    if i < remote_frame.len() {
                        remote_frame[i] = (r << 16) | (g << 8) | b;
                    }
                }
            }
        }

        // 5. Dimensiones reales del área cliente de la ventana
        #[cfg(windows)]
        let (client_w, client_h, pt_x, pt_y) = {
            let hwnd_ptr = window.get_window_handle();
            let mut pt = POINT::default();
            let mut rect = RECT::default();
            if !hwnd_ptr.is_null() {
                let hwnd = HWND(hwnd_ptr as _);
                unsafe {
                    let _ = GetCursorPos(&mut pt);
                    let _ = ScreenToClient(hwnd, &mut pt);
                    let _ = GetClientRect(hwnd, &mut rect);
                }
            }
            let cw = (rect.right - rect.left).max(400) as usize;
            let ch = (rect.bottom - rect.top).max(300) as usize;
            (cw, ch, pt.x as f32, pt.y as f32)
        };

        #[cfg(not(windows))]
        let (client_w, client_h, pt_x, pt_y) = {
            let (w, h) = window.get_size();
            let (mx, my) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap_or((0.0, 0.0));
            (w, h, mx, my)
        };

        // Redimensionar el buffer de dibujo exactamente al tamaño actual de la ventana cliente
        if buffer.len() != client_w * client_h {
            buffer.resize(client_w * client_h, 0x080C14);
        }

        // 6. Cálculo del área de trabajo para la imagen remota
        let area_top = if show_toolbar { TOOLBAR_HEIGHT } else { 0 };
        let area_h = client_h.saturating_sub(area_top);
        let area_w = client_w;

        let (draw_w, draw_h, off_x, off_y) = match current_scale_mode {
            ScaleAdaptation::AspectRatioFit => {
                let remote_ar = remote_w as f32 / remote_h.max(1) as f32;
                let area_ar = area_w as f32 / area_h.max(1) as f32;
                if area_ar > remote_ar {
                    let dh = area_h;
                    let dw = ((dh as f32) * remote_ar).round() as usize;
                    let ox = (area_w.saturating_sub(dw)) / 2;
                    let oy = area_top;
                    (dw.max(1), dh.max(1), ox, oy)
                } else {
                    let dw = area_w;
                    let dh = ((dw as f32) / remote_ar).round() as usize;
                    let ox = 0;
                    let oy = area_top + (area_h.saturating_sub(dh)) / 2;
                    (dw.max(1), dh.max(1), ox, oy)
                }
            }
            ScaleAdaptation::Stretch => {
                (area_w.max(1), area_h.max(1), 0, area_top)
            }
            ScaleAdaptation::Original100 => {
                let dw = remote_w.min(area_w).max(1);
                let dh = remote_h.min(area_h).max(1);
                let ox = (area_w.saturating_sub(dw)) / 2;
                let oy = area_top + (area_h.saturating_sub(dh)) / 2;
                (dw, dh, ox, oy)
            }
        };

        // 7. Blit y escalado del escritorio remoto sobre el buffer
        if current_scale_mode != ScaleAdaptation::Stretch {
            for y in area_top..client_h {
                let row_start = y * client_w;
                if y < off_y || y >= off_y + draw_h {
                    buffer[row_start..row_start + client_w].fill(0x060911);
                } else {
                    if off_x > 0 {
                        buffer[row_start..row_start + off_x].fill(0x060911);
                    }
                    let right_start = off_x + draw_w;
                    if right_start < client_w {
                        buffer[row_start + right_start..row_start + client_w].fill(0x060911);
                    }
                }
            }
        }

        if remote_w > 0 && remote_h > 0 && draw_w > 0 && draw_h > 0 {
            if draw_w == remote_w && draw_h == remote_h {
                for dy in 0..draw_h {
                    let dst_y = off_y + dy;
                    if dst_y >= client_h { break; }
                    let src_start = dy * remote_w;
                    let dst_start = dst_y * client_w + off_x;
                    buffer[dst_start..dst_start + draw_w].copy_from_slice(&remote_frame[src_start..src_start + draw_w]);
                }
            } else {
                let x_lut: Vec<usize> = (0..draw_w)
                    .map(|x| ((x * remote_w) / draw_w).min(remote_w - 1))
                    .collect();

                for dy in 0..draw_h {
                    let dst_y = off_y + dy;
                    if dst_y >= client_h { break; }
                    let src_y = ((dy * remote_h) / draw_h).min(remote_h - 1);
                    let src_row = &remote_frame[src_y * remote_w..];
                    let dst_start = dst_y * client_w + off_x;
                    let dst_row = &mut buffer[dst_start..dst_start + draw_w];

                    for (dst, &sx) in dst_row.iter_mut().zip(&x_lut) {
                        *dst = src_row[sx];
                    }
                }
            }
        }

        // 8. Botones de la barra superior (Coordenadas cliente exactas)
        let btn_calidad_rect = (10, 4, 180, 26);
        let btn_escala_rect = (196, 4, 170, 26);
        let btn_grabar_rect = (372, 4, 130, 26);
        let btn_win_rect = (508, 4, 80, 26);
        let btn_ocultar_rect = (594, 4, 95, 26);
        let btn_salir_rect = (695, 4, 80, 26);
        let btn_mostrar_rect = (10, 2, 75, 22);

        let is_in = |rect: (usize, usize, usize, usize)| {
            pt_x >= rect.0 as f32 && pt_x <= (rect.0 + rect.2) as f32
                && pt_y >= rect.1 as f32 && pt_y <= (rect.1 + rect.3) as f32
        };

        let is_over_toolbar = show_toolbar && pt_y >= 0.0 && pt_y < TOOLBAR_HEIGHT as f32;
        let is_over_mini_menu = !show_toolbar && is_in(btn_mostrar_rect);

        // 9. Detección de clics de mouse
        let is_window_active = window.is_active();
        let is_cursor_in_window = pt_x >= 0.0 && pt_x < client_w as f32 && pt_y >= 0.0 && pt_y < client_h as f32;
        let left_down_minifb = window.get_mouse_down(MouseButton::Left);
        let right_down_minifb = window.get_mouse_down(MouseButton::Right);

        #[cfg(windows)]
        let left_down_win32 = unsafe { GetAsyncKeyState(0x01) as u16 & 0x8000 != 0 };
        #[cfg(windows)]
        let right_down_win32 = unsafe { GetAsyncKeyState(0x02) as u16 & 0x8000 != 0 };

        #[cfg(not(windows))]
        let left_down_win32 = false;
        #[cfg(not(windows))]
        let right_down_win32 = false;

        let left_is_down = is_window_active && is_cursor_in_window && (left_down_minifb || left_down_win32);
        let right_is_down = is_window_active && is_cursor_in_window && (right_down_minifb || right_down_win32);

        let left_just_pressed = left_is_down && !left_was_down;
        let left_just_released = !left_is_down && left_was_down;

        // Normalización exacta respecto al área dibujada del escritorio remoto
        let norm_x = ((pt_x - off_x as f32) / draw_w as f32).clamp(0.0, 1.0);
        let norm_y = ((pt_y - off_y as f32) / draw_h as f32).clamp(0.0, 1.0);
        let is_over_remote_image = !is_over_toolbar && !is_over_mini_menu
            && pt_x >= off_x as f32 && pt_x < (off_x + draw_w) as f32
            && pt_y >= off_y as f32 && pt_y < (off_y + draw_h) as f32;

        if left_just_pressed {
            if show_toolbar && is_over_toolbar {
                if is_in(btn_calidad_rect) {
                    quality_tier = (quality_tier + 1) % 4;
                    let (q, lbl) = match quality_tier {
                        0 => (40, "Velocidad (40%)"),
                        1 => (70, "Equilibrada (70%)"),
                        2 => (88, "HD Nativa (88%)"),
                        _ => (96, "Ultra HD Cristalina (96%)"),
                    };
                    current_quality_label = lbl;
                    let _ = control_tx.send(ControlEvent::SetQuality { quality: q });
                } else if is_in(btn_escala_rect) {
                    current_scale_mode = match current_scale_mode {
                        ScaleAdaptation::AspectRatioFit => ScaleAdaptation::Stretch,
                        ScaleAdaptation::Stretch => ScaleAdaptation::Original100,
                        ScaleAdaptation::Original100 => ScaleAdaptation::AspectRatioFit,
                    };
                } else if is_in(btn_grabar_rect) {
                    if !recorder.is_recording {
                        let _ = recorder.start(title);
                    } else {
                        let _ = recorder.stop();
                    }
                } else if is_in(btn_win_rect) {
                    let _ = control_tx.send(ControlEvent::Keyboard { vk_code: 0x5B, down: true });
                    let _ = control_tx.send(ControlEvent::Keyboard { vk_code: 0x5B, down: false });
                } else if is_in(btn_ocultar_rect) {
                    show_toolbar = false;
                } else if is_in(btn_salir_rect) {
                    break;
                }
            } else if is_over_mini_menu {
                show_toolbar = true;
            } else if is_over_remote_image {
                let _ = control_tx.send(ControlEvent::MouseButton {
                    button: MouseButtonType::Left,
                    down: true,
                    x: Some(norm_x),
                    y: Some(norm_y),
                });
            }
        }

        if left_just_released {
            if !is_over_toolbar && !is_over_mini_menu {
                let _ = control_tx.send(ControlEvent::MouseButton {
                    button: MouseButtonType::Left,
                    down: false,
                    x: Some(norm_x),
                    y: Some(norm_y),
                });
            }
        }
        left_was_down = left_is_down;

        if right_is_down != right_was_down {
            right_was_down = right_is_down;
            if !is_over_toolbar && !is_over_mini_menu && is_over_remote_image {
                let _ = control_tx.send(ControlEvent::MouseButton {
                    button: MouseButtonType::Right,
                    down: right_is_down,
                    x: Some(norm_x),
                    y: Some(norm_y),
                });
            }
        }

        // Movimiento del cursor sobre el escritorio remoto
        if is_cursor_in_window && is_over_remote_image {
            let should_send = match last_mouse_pos {
                Some((lx, ly)) => (lx - norm_x).abs() > 0.0005 || (ly - norm_y).abs() > 0.0005,
                None => true,
            };

            if should_send {
                last_mouse_pos = Some((norm_x, norm_y));
                let _ = control_tx.send(ControlEvent::MouseMove {
                    x: norm_x,
                    y: norm_y,
                });
            }
        }

        // Rueda de desplazamiento (Scroll)
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            if scroll_y.abs() > 0.01 {
                let _ = control_tx.send(ControlEvent::MouseWheel {
                    delta_x: 0,
                    delta_y: scroll_y.round() as i32,
                });
            }
        }

        // 10. Renderizar la barra superior en la ventana
        if show_toolbar {
            for y in 0..TOOLBAR_HEIGHT {
                let row_start = y * client_w;
                buffer[row_start..row_start + client_w].fill(0x0B0F19);
            }
            if client_w > 0 && TOOLBAR_HEIGHT > 0 {
                let line_y = TOOLBAR_HEIGHT - 1;
                buffer[line_y * client_w..(line_y + 1) * client_w].fill(0x00E5FF);
            }

            let cal_label = format!("Calidad: {}", current_quality_label);
            draw_button(&mut buffer, client_w, client_h, btn_calidad_rect, &cal_label, is_in(btn_calidad_rect), false, 0);

            let esc_label = format!("Escala: {}", current_scale_mode.label());
            draw_button(&mut buffer, client_w, client_h, btn_escala_rect, &esc_label, is_in(btn_escala_rect), false, 0);

            let (rec_lbl, rec_col) = if recorder.is_recording {
                ("REC [Grabando]", 0xEF4444)
            } else {
                ("Grabar (F9)", 0)
            };
            draw_button(&mut buffer, client_w, client_h, btn_grabar_rect, rec_lbl, is_in(btn_grabar_rect), recorder.is_recording, rec_col);
            draw_button(&mut buffer, client_w, client_h, btn_win_rect, "Win Key", is_in(btn_win_rect), false, 0);
            draw_button(&mut buffer, client_w, client_h, btn_ocultar_rect, "Ocultar", is_in(btn_ocultar_rect), false, 0);
            draw_button(&mut buffer, client_w, client_h, btn_salir_rect, "Salir", is_in(btn_salir_rect), false, 0x991B1B);
        } else {
            draw_button(&mut buffer, client_w, client_h, btn_mostrar_rect, "Menu", is_over_mini_menu, false, 0);
        }

        // 11. Eventos de teclado con repetición continua (elimina continuamente al mantener pulsado Backspace)
        let pressed_keys = window.get_keys_pressed(KeyRepeat::Yes);
        for key in pressed_keys {
            if matches!(key, Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F9 | Key::Escape) {
                continue;
            }
            if let Some(vk_code) = minifb_key_to_vk(key) {
                let _ = control_tx.send(ControlEvent::Keyboard {
                    vk_code,
                    down: true,
                });
            }
        }

        let released_keys = window.get_keys_released();
        for key in released_keys {
            if matches!(key, Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F9 | Key::Escape) {
                continue;
            }
            if let Some(vk_code) = minifb_key_to_vk(key) {
                let _ = control_tx.send(ControlEvent::Keyboard {
                    vk_code,
                    down: false,
                });
            }
        }

        // 12. Título dinámico
        let rec_tag = if recorder.is_recording { " | 🔴 GRABANDO" } else { "" };
        let dynamic_title = format!("{} | {} | {} {}", title, current_quality_label, current_scale_mode.label(), rec_tag);
        window.set_title(&dynamic_title);

        let _ = window.update_with_buffer(&buffer, client_w, client_h);
    }

    #[cfg(windows)]
    if let Some(h) = hook {
        unsafe {
            let _ = UnhookWindowsHookEx(h);
        }
        HOOK_VIEWER_HWND.store(0, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut guard) = HOOK_TX.lock() {
            *guard = None;
        }
    }

    if recorder.is_recording {
        recorder.stop();
    }
}
