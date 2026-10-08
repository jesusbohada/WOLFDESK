use eframe::egui;

pub static EGUI_CTX: std::sync::Mutex<Option<egui::Context>> = std::sync::Mutex::new(None);

pub fn set_egui_context(ctx: egui::Context) {
    if let Ok(mut guard) = EGUI_CTX.lock() {
        *guard = Some(ctx);
    }
}

pub fn wake_ui() {
    if let Ok(guard) = EGUI_CTX.lock() {
        if let Some(ref ctx) = *guard {
            ctx.request_repaint();
        }
    }
}

#[cfg(windows)]
pub fn start_system_tray(my_id: &str, tx_to_ui: std::sync::mpsc::Sender<crate::dashboard::NetToUi>) {
    let id_clone = my_id.to_string();
    std::thread::spawn(move || {
        use windows::core::{w, PCWSTR};
        use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM, HINSTANCE};
        use windows::Win32::UI::Shell::{
            Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE, NIF_TIP,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
            DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW,
            RegisterClassW, SetForegroundWindow, TrackPopupMenuEx, TranslateMessage,
            HMENU, HICON, MSG, TPM_NONOTIFY, TPM_RETURNCMD,
            WNDCLASSW, WS_OVERLAPPED, CW_USEDEFAULT, IDI_APPLICATION,
            MF_SEPARATOR, MF_STRING, WM_APP, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP,
        };

        const WM_TRAYICON: u32 = WM_APP + 100;

        static TRAY_TX: std::sync::Mutex<Option<std::sync::mpsc::Sender<crate::dashboard::NetToUi>>> =
            std::sync::Mutex::new(None);

        if let Ok(mut guard) = TRAY_TX.lock() {
            *guard = Some(tx_to_ui);
        }

        unsafe extern "system" fn tray_proc(
            hwnd: HWND,
            msg: u32,
            _wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            if msg == WM_TRAYICON {
                let mouse_ev = lparam.0 as u32;
                if mouse_ev == WM_LBUTTONUP || mouse_ev == WM_LBUTTONDBLCLK {
                    if let Ok(guard) = TRAY_TX.lock() {
                        if let Some(ref tx) = *guard {
                            let _ = tx.send(crate::dashboard::NetToUi::ShowWindow);
                            wake_ui();
                        }
                    }
                } else if mouse_ev == WM_RBUTTONUP {
                    if let Ok(hmenu) = CreatePopupMenu() {
                        let _ = AppendMenuW(hmenu, MF_STRING, 1001, w!("🐺 Abrir WolfDesk"));
                        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(hmenu, MF_STRING, 1002, w!("🔴 Salir de WolfDesk"));

                        let mut pt = POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        let _ = SetForegroundWindow(hwnd);
                        let cmd = TrackPopupMenuEx(hmenu, (TPM_RETURNCMD | TPM_NONOTIFY).0, pt.x, pt.y, hwnd, None);
                        let _ = DestroyMenu(hmenu);

                        if cmd.0 == 1001 {
                            if let Ok(guard) = TRAY_TX.lock() {
                                if let Some(ref tx) = *guard {
                                    let _ = tx.send(crate::dashboard::NetToUi::ShowWindow);
                                    wake_ui();
                                }
                            }
                        } else if cmd.0 == 1002 {
                            let mut nid = NOTIFYICONDATAW::default();
                            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                            nid.hWnd = hwnd;
                            nid.uID = 1;
                            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
                            std::process::exit(0);
                        }
                    }
                }
                return LRESULT(0);
            }
            DefWindowProcW(hwnd, msg, _wparam, lparam)
        }

        unsafe {
            let class_name = w!("WolfDeskTrayClass");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_proc),
                lpszClassName: class_name,
                hInstance: HINSTANCE(0),
                ..Default::default()
            };
            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                class_name,
                w!("WolfDesk Tray Host"),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND(0),
                HMENU(0),
                HINSTANCE(0),
                None,
            );

            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = LoadIconW(HINSTANCE(0), IDI_APPLICATION).unwrap_or(HICON(0));

            let tip_text = format!("🐺 WolfDesk Pro [{}] (Activo en segundo plano)", id_clone);
            let mut tip_utf16: Vec<u16> = tip_text.encode_utf16().collect();
            tip_utf16.truncate(127);
            tip_utf16.push(0);
            for (i, &ch) in tip_utf16.iter().enumerate() {
                if i < nid.szTip.len() {
                    nid.szTip[i] = ch;
                }
            }

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
            log::info!("🐺 [TRAY WINDOWS] Icono en la bandeja del sistema registrado correctamente.");

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
}

#[cfg(target_os = "linux")]
fn get_tray_icon_pixmap() -> Vec<ksni::Icon> {
    let png_bytes = include_bytes!("../assets/wolfdesk.png");
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let mut argb_data = Vec::with_capacity((w * h * 4) as usize);
        for pixel in rgba.pixels() {
            // ksni::Icon data expects ARGB32 in network byte order (big endian):
            // byte 0: Alpha, byte 1: Red, byte 2: Green, byte 3: Blue
            argb_data.push(pixel[3]); // A
            argb_data.push(pixel[0]); // R
            argb_data.push(pixel[1]); // G
            argb_data.push(pixel[2]); // B
        }
        vec![ksni::Icon {
            width: w as i32,
            height: h as i32,
            data: argb_data,
        }]
    } else {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
pub fn start_system_tray(my_id: &str, tx_to_ui: std::sync::mpsc::Sender<crate::dashboard::NetToUi>) {
    let id_clone = my_id.to_string();
    std::thread::spawn(move || {
        // 1. Escribir icono a disco para que el tema del escritorio o cargador lo encuentre siempre
        let icon_bytes = include_bytes!("../assets/wolfdesk.png");
        let _ = std::fs::write("/tmp/wolfdesk.png", icon_bytes);
        let _ = std::fs::write("/tmp/preferences-desktop-remote-desktop.png", icon_bytes);
        let _ = std::fs::write("/usr/share/pixmaps/wolfdesk.png", icon_bytes);

        // 2. Asegurar que root tenga permiso en el bus de sesión D-Bus de /etc/dbus-1
        let dbus_dir = std::path::Path::new("/etc/dbus-1");
        if dbus_dir.exists() {
            let conf_content = "<busconfig>\n  <policy context=\"mandatory\">\n    <allow user=\"root\"/>\n  </policy>\n</busconfig>\n";
            let _ = std::fs::write(dbus_dir.join("session-local.conf"), conf_content);
            let session_d = dbus_dir.join("session.d");
            let _ = std::fs::create_dir_all(&session_d);
            let _ = std::fs::write(session_d.join("allow-root.conf"), conf_content);
        }

        // 3. Autodetectar el bus de sesión D-Bus del usuario del escritorio (KDE Plasma / caja)
        let mut dbus_address = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
        if dbus_address.as_deref().unwrap_or("").trim().is_empty() {
            if std::path::Path::new("/run/user/1000/bus").exists() {
                dbus_address = Some("unix:path=/run/user/1000/bus".to_string());
            } else if std::path::Path::new("/run/user/1001/bus").exists() {
                dbus_address = Some("unix:path=/run/user/1001/bus".to_string());
            } else if let Ok(entries) = std::fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if let Ok(cmdline) = std::fs::read_to_string(p.join("cmdline")) {
                        if cmdline.contains("plasma") || cmdline.contains("kded") || cmdline.contains("kwin") {
                            if let Ok(env_bytes) = std::fs::read(p.join("environ")) {
                                for chunk in env_bytes.split(|&b| b == 0) {
                                    if let Ok(var_str) = std::str::from_utf8(chunk) {
                                        if let Some(stripped) = var_str.strip_prefix("DBUS_SESSION_BUS_ADDRESS=") {
                                            if !stripped.trim().is_empty() {
                                                dbus_address = Some(stripped.to_string());
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if dbus_address.is_some() {
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Some(ref addr) = dbus_address {
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", addr);
            log::info!("🐺 [TRAY LINUX] Bus de sesión D-Bus configurado: {}", addr);
        } else {
            log::warn!("⚠️ [TRAY LINUX] No se pudo encontrar DBUS_SESSION_BUS_ADDRESS.");
        }

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                log::error!("⚠️ [TRAY LINUX] Error al iniciar runtime Tokio para el tray: {:?}", e);
                return;
            }
        };

        rt.block_on(async move {
            use ksni::TrayMethods;
            let tray = WolfDeskLinuxTray {
                tx_to_ui,
                my_id: id_clone,
            };
            match tray.spawn().await {
                Ok(handle) => {
                    log::info!("🐺 [TRAY LINUX] ✅ Icono de bandeja del sistema (KDE Plasma) iniciado con éxito.");
                    std::mem::forget(handle);
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    }
                }
                Err(e) => {
                    log::warn!("⚠️ [TRAY LINUX] No se pudo inicializar la bandeja del sistema StatusNotifierItem: {:?}", e);
                }
            }
        });
    });
}

#[cfg(target_os = "linux")]
struct WolfDeskLinuxTray {
    tx_to_ui: std::sync::mpsc::Sender<crate::dashboard::NetToUi>,
    my_id: String,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for WolfDeskLinuxTray {
    fn id(&self) -> String {
        "wolfdesk".into()
    }

    fn category(&self) -> ksni::Category {
        ksni::Category::ApplicationStatus
    }

    fn title(&self) -> String {
        format!("WolfDesk Pro [{}]", self.my_id)
    }

    fn status(&self) -> ksni::Status {
        ksni::Status::Active
    }

    fn icon_name(&self) -> String {
        "preferences-desktop-remote-desktop".into()
    }

    fn icon_theme_path(&self) -> String {
        "/tmp".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        get_tray_icon_pixmap()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "WolfDesk Pro".into(),
            description: format!("ID: {} | Servicio activo en segundo plano", self.my_id),
            icon_name: "preferences-desktop-remote-desktop".into(),
            icon_pixmap: get_tray_icon_pixmap(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        log::info!("🐺 [TRAY LINUX] Clic en icono de bandeja. Restaurando ventana...");
        let _ = self.tx_to_ui.send(crate::dashboard::NetToUi::ShowWindow);
        wake_ui();
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx_to_ui.send(crate::dashboard::NetToUi::ShowWindow);
        wake_ui();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "🐺 Abrir WolfDesk".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.tx_to_ui.send(crate::dashboard::NetToUi::ShowWindow);
                    wake_ui();
                }),
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: format!("🆔 ID: {}", self.my_id),
                enabled: false,
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "🔴 Salir de WolfDesk".into(),
                activate: Box::new(|_| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn start_system_tray(_my_id: &str, _tx_to_ui: std::sync::mpsc::Sender<crate::dashboard::NetToUi>) {}
