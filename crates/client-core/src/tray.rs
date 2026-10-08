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
pub fn start_system_tray(my_id: &str, tx_to_ui: std::sync::mpsc::Sender<crate::dashboard::NetToUi>) {
    let id_clone = my_id.to_string();
    std::thread::spawn(move || {
        // Asegurar que el proceso root tenga acceso al bus de sesión del usuario de escritorio (KDE Plasma)
        if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
            let candidates = [
                "/run/user/1000/bus",
                "/run/user/1001/bus",
            ];
            for path in &candidates {
                if std::path::Path::new(path).exists() {
                    std::env::set_var("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", path));
                    break;
                }
            }
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
                    log::info!("🐺 [TRAY LINUX] Icono de bandeja del sistema (KDE Plasma) iniciado con éxito.");
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

    fn title(&self) -> String {
        format!("WolfDesk Pro [{}]", self.my_id)
    }

    fn icon_name(&self) -> String {
        "network-workgroup".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "WolfDesk Pro".into(),
            description: format!("ID: {} | Servicio activo en segundo plano", self.my_id),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        log::info!("🐺 [TRAY LINUX] Clic en icono de bandeja. Restaurando ventana...");
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
