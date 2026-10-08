use proto::{ControlEvent, MouseButtonType, SessionPermissions};

#[cfg(windows)]
use windows::Win32::{
    UI::Input::KeyboardAndMouse::{
        MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
        KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAP_VIRTUAL_KEY_TYPE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
        MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, VIRTUAL_KEY,
    },
    UI::WindowsAndMessaging::{GetSystemMetrics, SetCursorPos, SM_CXSCREEN, SM_CYSCREEN},
};

/// Procesa un evento de control remoto verificando los permisos de seguridad concedidos por el Host
pub fn dispatch_event_with_permissions(event: &ControlEvent, permissions: &SessionPermissions) {
    #[cfg(windows)]
    match event {
        ControlEvent::MouseMove { x, y } => {
            if !permissions.allow_mouse {
                return;
            }
            unsafe {
                let hdc_screen = windows::Win32::Graphics::Gdi::GetDC(windows::Win32::Foundation::HWND(0));
                let phys_w = windows::Win32::Graphics::Gdi::GetDeviceCaps(hdc_screen, windows::Win32::Graphics::Gdi::GET_DEVICE_CAPS_INDEX(118));
                let phys_h = windows::Win32::Graphics::Gdi::GetDeviceCaps(hdc_screen, windows::Win32::Graphics::Gdi::GET_DEVICE_CAPS_INDEX(117));
                let _ = windows::Win32::Graphics::Gdi::ReleaseDC(windows::Win32::Foundation::HWND(0), hdc_screen);

                let screen_w = if phys_w > 0 { phys_w } else { GetSystemMetrics(SM_CXSCREEN) };
                let screen_h = if phys_h > 0 { phys_h } else { GetSystemMetrics(SM_CYSCREEN) };

                let target_x = (x.clamp(0.0, 1.0) * (screen_w - 1) as f32).round() as i32;
                let target_y = (y.clamp(0.0, 1.0) * (screen_h - 1) as f32).round() as i32;
                let _ = SetCursorPos(target_x, target_y);
            }
        }

        ControlEvent::MouseButton { button, down, x, y } => {
            if !permissions.allow_mouse {
                return;
            }
            unsafe {
                if let (Some(px), Some(py)) = (x, y) {
                    let hdc_screen = windows::Win32::Graphics::Gdi::GetDC(windows::Win32::Foundation::HWND(0));
                    let phys_w = windows::Win32::Graphics::Gdi::GetDeviceCaps(hdc_screen, windows::Win32::Graphics::Gdi::GET_DEVICE_CAPS_INDEX(118));
                    let phys_h = windows::Win32::Graphics::Gdi::GetDeviceCaps(hdc_screen, windows::Win32::Graphics::Gdi::GET_DEVICE_CAPS_INDEX(117));
                    let _ = windows::Win32::Graphics::Gdi::ReleaseDC(windows::Win32::Foundation::HWND(0), hdc_screen);

                    let screen_w = if phys_w > 0 { phys_w } else { GetSystemMetrics(SM_CXSCREEN) };
                    let screen_h = if phys_h > 0 { phys_h } else { GetSystemMetrics(SM_CYSCREEN) };

                    let target_x = (px.clamp(0.0, 1.0) * (screen_w - 1) as f32).round() as i32;
                    let target_y = (py.clamp(0.0, 1.0) * (screen_h - 1) as f32).round() as i32;
                    let _ = SetCursorPos(target_x, target_y);
                }

                let flag = match (button, down) {
                    (MouseButtonType::Left, true) => MOUSEEVENTF_LEFTDOWN,
                    (MouseButtonType::Left, false) => MOUSEEVENTF_LEFTUP,
                    (MouseButtonType::Right, true) => MOUSEEVENTF_RIGHTDOWN,
                    (MouseButtonType::Right, false) => MOUSEEVENTF_RIGHTUP,
                    (MouseButtonType::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
                    (MouseButtonType::Middle, false) => MOUSEEVENTF_MIDDLEUP,
                };

                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: flag,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }

        ControlEvent::MouseWheel { delta_y, .. } => {
            if !permissions.allow_mouse {
                return;
            }
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: (*delta_y * 120) as u32,
                        dwFlags: MOUSEEVENTF_WHEEL,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            unsafe {
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }

        ControlEvent::Keyboard { vk_code, down } => {
            if !permissions.allow_keyboard {
                return;
            }
            #[cfg(windows)]
            {
                static mut HOST_KEY_STATES: [bool; 256] = [false; 256];
                let is_extended = matches!(*vk_code, 0x5B | 0x5C | 0x5D | 0x21..=0x28 | 0x2D | 0x2E);
                let ext_flag = if is_extended { KEYEVENTF_EXTENDEDKEY } else { windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0) };
                let scan = unsafe { MapVirtualKeyW(*vk_code as u32, MAP_VIRTUAL_KEY_TYPE(0)) } as u16;
                let vk_idx = (*vk_code as usize) & 0xFF;
                let is_modifier = matches!(*vk_code, 0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0x5D | 0x14 | 0x90);

                unsafe {
                    if *down {
                        let was_down = HOST_KEY_STATES[vk_idx];
                        HOST_KEY_STATES[vk_idx] = true;

                        if was_down && !is_modifier {
                            // Repetición automática de tecla mantenida (ej. Backspace continuo)
                            let inputs = [
                                INPUT {
                                    r#type: INPUT_KEYBOARD,
                                    Anonymous: INPUT_0 {
                                        ki: KEYBDINPUT {
                                            wVk: VIRTUAL_KEY(*vk_code),
                                            wScan: scan,
                                            dwFlags: KEYEVENTF_KEYUP | ext_flag,
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                },
                                INPUT {
                                    r#type: INPUT_KEYBOARD,
                                    Anonymous: INPUT_0 {
                                        ki: KEYBDINPUT {
                                            wVk: VIRTUAL_KEY(*vk_code),
                                            wScan: scan,
                                            dwFlags: ext_flag,
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                },
                            ];
                            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                        } else {
                            let input = INPUT {
                                r#type: INPUT_KEYBOARD,
                                Anonymous: INPUT_0 {
                                    ki: KEYBDINPUT {
                                        wVk: VIRTUAL_KEY(*vk_code),
                                        wScan: scan,
                                        dwFlags: ext_flag,
                                        time: 0,
                                        dwExtraInfo: 0,
                                    },
                                },
                            };
                            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                        }
                    } else {
                        HOST_KEY_STATES[vk_idx] = false;
                        let input = INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 {
                                ki: KEYBDINPUT {
                                    wVk: VIRTUAL_KEY(*vk_code),
                                    wScan: scan,
                                    dwFlags: KEYEVENTF_KEYUP | ext_flag,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                }
            }
        }

        ControlEvent::ClipboardSync { text } => {
            if !permissions.allow_clipboard {
                return;
            }
            log::info!("Sincronización de portapapeles autorizada: {} caracteres", text.len());
        }

        ControlEvent::SetQuality { .. } | ControlEvent::Ping { .. } | ControlEvent::Pong { .. } => {}
    }

    #[cfg(target_os = "linux")]
    {
        dispatch_linux_input_event(event, permissions);
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = (event, permissions);
    }
}

#[cfg(target_os = "linux")]
struct LinuxInputContext {
    conn: x11rb::rust_connection::RustConnection,
    root: u32,
    screen_w: f32,
    screen_h: f32,
}

#[cfg(target_os = "linux")]
static LINUX_INPUT_CTX: std::sync::Mutex<Option<LinuxInputContext>> = std::sync::Mutex::new(None);

#[cfg(target_os = "linux")]
fn dispatch_linux_input_event(event: &ControlEvent, permissions: &SessionPermissions) {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::ConnectionExt as XProtoExt;
    use x11rb::protocol::xtest::ConnectionExt as XTestExt;

    crate::capture::ensure_linux_x11_auth();

    let mut guard = match LINUX_INPUT_CTX.lock() {
        Ok(g) => g,
        Err(_) => return,
    };

    if guard.is_none() {
        if std::env::var("DISPLAY").is_err() {
            std::env::set_var("DISPLAY", ":0");
        }

        let (conn, screen_num) = match x11rb::connect(None) {
            Ok(c) => c,
            Err(e) => {
                log::error!("❌ [INPUT X11] Error al conectar con X11 para inyección de periféricos: {:?}", e);
                return;
            }
        };

        let setup = conn.setup();
        let screen = match setup.roots.get(screen_num) {
            Some(s) => s,
            None => {
                log::error!("❌ [INPUT X11] Pantalla X11 no encontrada (screen_num={})", screen_num);
                return;
            }
        };

        let root = screen.root;
        let mut scr_w = screen.width_in_pixels as f32;
        let mut scr_h = screen.height_in_pixels as f32;

        if let Ok(geom_cookie) = conn.get_geometry(root) {
            if let Ok(geom) = geom_cookie.reply() {
                scr_w = geom.width as f32;
                scr_h = geom.height as f32;
            }
        }

        log::info!("🎮 [INPUT X11] Conexión XTEST persistente inicializada con éxito (Pantalla: {}x{}, Root: 0x{:x})", scr_w, scr_h, root);
        *guard = Some(LinuxInputContext {
            conn,
            root,
            screen_w: scr_w,
            screen_h: scr_h,
        });
    }

    let ctx = match guard.as_mut() {
        Some(c) => c,
        None => return,
    };

    match event {
        ControlEvent::MouseMove { x, y } => {
            if !permissions.allow_mouse {
                return;
            }
            let target_x = (x.clamp(0.0, 1.0) * (ctx.screen_w - 1.0)).round() as i16;
            let target_y = (y.clamp(0.0, 1.0) * (ctx.screen_h - 1.0)).round() as i16;

            // 1. Mover el cursor directamente en el servidor X11
            let _ = ctx.conn.warp_pointer(x11rb::NONE, ctx.root, 0, 0, 0, 0, target_x, target_y);
            // 2. Enviar evento MotionNotify sintético por XTEST (Tipo 6) para actualizar ventanas y hover
            let _ = ctx.conn.xtest_fake_input(6, 0, 0, ctx.root, target_x, target_y, 0);
            let _ = ctx.conn.flush();
        }

        ControlEvent::MouseButton { button, down, x, y } => {
            if !permissions.allow_mouse {
                return;
            }
            if let (Some(px), Some(py)) = (x, y) {
                let target_x = (px.clamp(0.0, 1.0) * (ctx.screen_w - 1.0)).round() as i16;
                let target_y = (py.clamp(0.0, 1.0) * (ctx.screen_h - 1.0)).round() as i16;
                let _ = ctx.conn.warp_pointer(x11rb::NONE, ctx.root, 0, 0, 0, 0, target_x, target_y);
                let _ = ctx.conn.xtest_fake_input(6, 0, 0, ctx.root, target_x, target_y, 0);
            }

            let btn_code = match button {
                MouseButtonType::Left => 1,
                MouseButtonType::Middle => 2,
                MouseButtonType::Right => 3,
            };
            let ev_type = if *down { 4 } else { 5 }; // 4 = ButtonPress, 5 = ButtonRelease
            // En XTEST para ButtonPress/ButtonRelease el root DEBE ser x11rb::NONE (0)
            let res = ctx.conn.xtest_fake_input(ev_type, btn_code, 0, x11rb::NONE, 0, 0, 0);
            let _ = ctx.conn.flush();
            log::info!("🖱️ [INPUT CLIC] Botón {:?} (down={}) -> Éxito: {}", button, down, res.is_ok());
        }

        ControlEvent::MouseWheel { delta_y, .. } => {
            if !permissions.allow_mouse {
                return;
            }
            let btn = if *delta_y > 0 { 4 } else { 5 }; // 4 = WheelUp, 5 = WheelDown
            let _ = ctx.conn.xtest_fake_input(4, btn, 0, x11rb::NONE, 0, 0, 0);
            let _ = ctx.conn.xtest_fake_input(5, btn, 0, x11rb::NONE, 0, 0, 0);
            let _ = ctx.conn.flush();
        }

        ControlEvent::Keyboard { vk_code, down } => {
            if !permissions.allow_keyboard {
                return;
            }
            if let Some(keycode) = map_vk_to_linux_keycode(*vk_code) {
                let ev_type = if *down { 2 } else { 3 }; // 2 = KeyPress, 3 = KeyRelease
                // En XTEST para KeyPress/KeyRelease el root DEBE ser x11rb::NONE (0)
                let res = ctx.conn.xtest_fake_input(ev_type, keycode, 0, x11rb::NONE, 0, 0, 0);
                let _ = ctx.conn.flush();
                log::info!("⌨️ [INPUT TECLADO] VK 0x{:X} -> Keycode {} (down={}) -> Éxito: {}", vk_code, keycode, down, res.is_ok());
            } else {
                log::warn!("⚠️ [INPUT TECLADO] Tecla VK 0x{:X} no mapeada a código de tecla Linux", vk_code);
            }
        }

        _ => {}
    }
}

#[cfg(target_os = "linux")]
fn map_vk_to_linux_keycode(vk: u16) -> Option<u8> {
    match vk {
        0x08 => Some(22),  // BackSpace
        0x09 => Some(23),  // Tab
        0x0D => Some(36),  // Enter / Return
        0x1B => Some(9),   // Escape
        0x20 => Some(65),  // Space
        0x21 => Some(112), // PageUp
        0x22 => Some(117), // PageDown
        0x23 => Some(115), // End
        0x24 => Some(110), // Home
        0x25 => Some(113), // Left
        0x26 => Some(111), // Up
        0x27 => Some(114), // Right
        0x28 => Some(116), // Down
        0x2D => Some(118), // Insert
        0x2E => Some(119), // Delete
        0x10 | 0xA0 => Some(50), // LShift
        0xA1 => Some(62),        // RShift
        0x11 | 0xA2 => Some(37), // LControl
        0xA3 => Some(105),       // RControl
        0x12 | 0xA4 => Some(64), // LAlt
        0xA5 => Some(108),       // RAlt / AltGr
        0x5B => Some(133),       // Super_L (Win)
        0x5C => Some(134),       // Super_R (Win)

        // Números 0-9
        0x30 => Some(19), // 0
        0x31..=0x39 => Some((vk - 0x31 + 10) as u8), // 1-9 -> 10-18

        // Teclado numérico (Numpad 0-9)
        0x60 => Some(90), // KP_0
        0x61..=0x69 => Some((vk - 0x61 + 87) as u8), // KP_1..KP_9 -> 87..95
        0x6A => Some(63), // KP_Multiply
        0x6B => Some(86), // KP_Add
        0x6D => Some(82), // KP_Subtract
        0x6E => Some(91), // KP_Decimal
        0x6F => Some(106), // KP_Divide

        // Letras A-Z
        0x41 => Some(38), // A
        0x42 => Some(56), // B
        0x43 => Some(54), // C
        0x44 => Some(40), // D
        0x45 => Some(26), // E
        0x46 => Some(41), // F
        0x47 => Some(42), // G
        0x48 => Some(43), // H
        0x49 => Some(31), // I
        0x4A => Some(44), // J
        0x4B => Some(45), // K
        0x4C => Some(46), // L
        0x4D => Some(58), // M
        0x4E => Some(57), // N
        0x4F => Some(32), // O
        0x50 => Some(33), // P
        0x51 => Some(24), // Q
        0x52 => Some(27), // R
        0x53 => Some(39), // S
        0x54 => Some(28), // T
        0x55 => Some(30), // U
        0x56 => Some(55), // V
        0x57 => Some(25), // W
        0x58 => Some(53), // X
        0x59 => Some(29), // Y
        0x5A => Some(52), // Z

        // Teclas F1-F12
        0x70..=0x79 => Some((vk - 0x70 + 67) as u8), // F1-F10 -> 67-76
        0x7A => Some(95),                            // F11
        0x7B => Some(96),                            // F12

        // Puntuación básica
        0xBA => Some(47), // Semicolon
        0xBB => Some(21), // Equal
        0xBC => Some(59), // Comma
        0xBD => Some(20), // Minus
        0xBE => Some(60), // Period
        0xBF => Some(61), // Slash
        0xC0 => Some(48), // Grave / Tilde
        0xDB => Some(34), // BracketLeft
        0xDC => Some(51), // Backslash
        0xDD => Some(35), // BracketRight
        0xDE => Some(49), // Quote / Apostrophe

        _ => None,
    }
}
