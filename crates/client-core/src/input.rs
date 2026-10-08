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

    #[cfg(not(windows))]
    {
        let _ = (event, permissions);
    }
}
