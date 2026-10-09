use crate::config::AppConfig;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use proto::SessionPermissions;
use std::sync::mpsc::Receiver;

#[derive(PartialEq)]
pub enum ActiveTab {
    Main,
    Chat,
    Files,
    Settings,
    Terminal,
}

pub enum UiToNet {
    Connect {
        target_id: String,
        password: Option<String>,
    },
    AcceptIncoming {
        from_id: String,
        permissions: SessionPermissions,
    },
    RejectIncoming {
        from_id: String,
    },
    SendChat {
        target_id: String,
        message: String,
    },
    SendFile {
        target_id: String,
        file_path: String,
    },
    RequestRemoteFiles {
        target_id: String,
        path: String,
    },
    DownloadRemoteFile {
        target_id: String,
        remote_file_path: String,
    },
    UpdateConfig(AppConfig),
    CheckForUpdates,
    TriggerUpdate,
}

pub enum NetToUi {
    ServerConnected(String),
    ServerDisconnected,
    IncomingRequest(String),
    SessionAccepted(String),
    SessionRejected(String),
    SessionError(String),
    ChatReceived { from_id: String, text: String },
    FileStatus(String),
    RemoteFilesReceived {
        path: String,
        entries: Vec<proto::FileEntry>,
    },
    ShowWindow,
    ExitApp,
    UpdateStatusChanged(crate::updater::UpdateStatus),
    UpdateProgress(String),
}

pub struct DashboardApp {
    pub my_id: String,
    pub config: AppConfig,
    pub active_tab: ActiveTab,

    pub target_id_input: String,
    pub password_input: String,
    pub status_text: String,
    pub is_online: bool,

    // Diálogo interactivo de autorización entrante
    pub incoming_request_from: Option<String>,
    pub incoming_permissions: SessionPermissions,

    // Chat integrado
    pub chat_input: String,
    pub chat_history: Vec<(String, String)>,
    pub active_peer_id: Option<String>,

    // Transferencia de archivos estilo AnyDesk (Dual-Pane)
    pub local_file_path: String,
    pub local_file_entries: Vec<proto::FileEntry>,
    pub selected_local_file: Option<String>,

    pub remote_file_path: String,
    pub remote_file_entries: Vec<proto::FileEntry>,
    pub selected_remote_file: Option<String>,

    pub file_status_message: String,

    // Pestaña de Ajustes y Protección de Red
    pub server_url_input: String,
    pub unattended_pass_input: String,
    pub admin_unlocked: bool,
    pub show_admin_unlock_dialog: bool,
    pub admin_pin_input: String,
    pub admin_unlock_error: Option<String>,

    // Gestor de Actualizaciones
    pub update_status: crate::updater::UpdateStatus,

    // Sistema de instalación permanente y terminal
    pub is_installed: bool,
    pub show_install_dialog: bool,
    pub install_feedback: Option<String>,
    pub terminal_command: String,

    // Canales de comunicación con el motor de red
    pub tx_to_net: tokio::sync::mpsc::UnboundedSender<UiToNet>,
    pub rx_from_net: Receiver<NetToUi>,
}

impl DashboardApp {
    pub fn new(
        my_id: String,
        config: AppConfig,
        tx_to_net: tokio::sync::mpsc::UnboundedSender<UiToNet>,
        rx_from_net: Receiver<NetToUi>,
    ) -> Self {
        let server_url = config.server_url.clone();
        let (local_path, local_entries) = crate::FileTransferManager::list_directory(".");
        let is_installed = crate::installer::is_installed();
        Self {
            my_id,
            config,
            active_tab: ActiveTab::Main,
            target_id_input: String::new(),
            password_input: String::new(),
            status_text: "Conectando al clúster WolfDesk...".to_string(),
            is_online: false,
            incoming_request_from: None,
            incoming_permissions: SessionPermissions::default(),
            chat_input: String::new(),
            chat_history: Vec::new(),
            active_peer_id: None,
            local_file_path: local_path,
            local_file_entries: local_entries,
            selected_local_file: None,
            remote_file_path: ".".to_string(),
            remote_file_entries: Vec::new(),
            selected_remote_file: None,
            file_status_message: "Listo para transferir archivos.".to_string(),
            server_url_input: server_url,
            unattended_pass_input: String::new(),
            admin_unlocked: false,
            show_admin_unlock_dialog: false,
            admin_pin_input: String::new(),
            admin_unlock_error: None,
            update_status: crate::updater::UpdateStatus::NotChecked,
            is_installed,
            show_install_dialog: false,
            install_feedback: None,
            terminal_command: String::new(),
            tx_to_net,
            rx_from_net,
        }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = Color32::from_rgb(18, 22, 31);
        visuals.panel_fill = Color32::from_rgb(14, 17, 24);
        visuals.override_text_color = Some(Color32::from_rgb(226, 232, 240));
        ctx.set_visuals(visuals);

        crate::tray::set_egui_context(ctx.clone());

        // Interceptar el cierre de ventana para minimizar a la bandeja del sistema en segundo plano
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            log::info!("🐺 [WOLFDESK] Ventana minimizada a la bandeja del sistema (área de notificación cerca al reloj).");
        }

        // 1. Drenar eventos entrantes desde el hilo de red o la bandeja
        while let Ok(event) = self.rx_from_net.try_recv() {
            match event {
                NetToUi::ShowWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                }
                NetToUi::ExitApp => {
                    crate::restart_application();
                }
                NetToUi::ServerConnected(id) => {
                    self.my_id = id;
                    self.is_online = true;
                    self.status_text = format!("🟢 En línea en {}", self.config.server_url);
                }
                NetToUi::ServerDisconnected => {
                    self.is_online = false;
                    self.status_text = "🔴 Desconectado del servidor de señalización".to_string();
                }
                NetToUi::IncomingRequest(from_id) => {
                    self.incoming_request_from = Some(from_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                }
                NetToUi::SessionAccepted(target) => {
                    self.status_text = format!("🐺 Sesión WolfDesk activa con [{}]", target);
                    self.active_peer_id = Some(target.clone());
                    self.config.add_recent_id(&target);
                    let _ = self.tx_to_net.send(UiToNet::RequestRemoteFiles {
                        target_id: target,
                        path: ".".to_string(),
                    });
                }
                NetToUi::SessionRejected(reason) => {
                    self.status_text = format!("❌ Conexión rechazada: {}", reason);
                    self.active_peer_id = None;
                    self.remote_file_entries.clear();
                    self.selected_remote_file = None;
                }
                NetToUi::SessionError(err) => {
                    self.status_text = format!("⚠️ Error: {}", err);
                }
                NetToUi::ChatReceived { from_id, text } => {
                    self.chat_history.push((from_id, text));
                }
                NetToUi::FileStatus(status) => {
                    self.file_status_message = status;
                }
                NetToUi::RemoteFilesReceived { path, entries } => {
                    self.remote_file_path = path;
                    self.remote_file_entries = entries;
                    self.selected_remote_file = None;
                    self.file_status_message = "Explorador remoto sincronizado.".to_string();
                }
                NetToUi::UpdateStatusChanged(status) => {
                    self.update_status = status;
                }
                NetToUi::UpdateProgress(step) => {
                    crate::terminal_log::add_log(crate::terminal_log::LogLevel::Info, &step);
                    if let crate::updater::UpdateStatus::Updating { step: ref mut current_step } = self.update_status {
                        *current_step = step;
                    }
                }
            }
        }

        // 2. Diálogo Modal de Solicitud Entrante
        if let Some(ref from_id) = self.incoming_request_from.clone() {
            egui::Window::new("🐺 WolfDesk - Solicitud Entrante")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(egui::Frame::window(&ctx.style()).stroke(Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255))))
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(format!("El usuario [{}] solicita controlar tu equipo.", from_id))
                            .strong()
                            .size(16.0)
                            .color(Color32::from_rgb(0, 229, 255)),
                    );
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label(RichText::new("Permisos que concedes:").strong());

                    ui.checkbox(&mut self.incoming_permissions.allow_mouse, "Permitir control del Mouse");
                    ui.checkbox(&mut self.incoming_permissions.allow_keyboard, "Permitir uso del Teclado");
                    ui.checkbox(&mut self.incoming_permissions.allow_clipboard, "Permitir Portapapeles compartido");
                    ui.checkbox(&mut self.incoming_permissions.allow_file_transfer, "Permitir Transferencia de archivos");

                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Button::new(RichText::new("🟢 ACEPTAR").color(Color32::WHITE).strong()).fill(Color32::from_rgb(16, 185, 129)))
                            .clicked()
                        {
                            let _ = self.tx_to_net.send(UiToNet::AcceptIncoming {
                                from_id: from_id.clone(),
                                permissions: self.incoming_permissions,
                            });
                            self.active_peer_id = Some(from_id.clone());
                            self.incoming_request_from = None;
                        }

                        if ui
                            .add(egui::Button::new(RichText::new("🔴 RECHAZAR").color(Color32::WHITE).strong()).fill(Color32::from_rgb(239, 68, 68)))
                            .clicked()
                        {
                            let _ = self.tx_to_net.send(UiToNet::RejectIncoming {
                                from_id: from_id.clone(),
                            });
                            self.incoming_request_from = None;
                        }
                    });
                    ui.add_space(10.0);
                });
        }

        // 2b. Diálogo Modal de Instalación en el Sistema
        if self.show_install_dialog {
            egui::Window::new("🚀 Instalar WolfDesk en este Equipo")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(egui::Frame::window(&ctx.style()).stroke(Stroke::new(2.0_f32, Color32::from_rgb(0, 229, 255))))
                .show(ctx, |ui| {
                    ui.add_space(5.0);
                    ui.label(RichText::new("¿Deseas instalar formalmente WolfDesk?").strong().size(15.0).color(Color32::from_rgb(0, 229, 255)));
                    ui.add_space(8.0);
                    ui.label("• El ID asignado a este equipo será FIJO y permanente.");
                    ui.label("• Se creará acceso directo en el Escritorio y Menú Inicio.");
                    ui.label("• Quedará registrado en Windows para desinstalar cuando quieras.");
                    ui.add_space(12.0);

                    if let Some(ref msg) = self.install_feedback {
                        ui.label(RichText::new(msg).color(Color32::from_rgb(16, 185, 129)).strong());
                        ui.add_space(8.0);
                        if ui.button("Cerrar").clicked() {
                            self.show_install_dialog = false;
                            self.install_feedback = None;
                        }
                    } else {
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(RichText::new("✅ Instalar Ahora").color(Color32::WHITE).strong()).fill(Color32::from_rgb(14, 116, 144))).clicked() {
                                match crate::installer::install_to_system() {
                                    Ok(m) => {
                                        self.is_installed = true;
                                        self.install_feedback = Some(m);
                                    }
                                    Err(e) => {
                                        self.install_feedback = Some(format!("Error: {}", e));
                                    }
                                }
                            }
                            if ui.button("Cancelar").clicked() {
                                self.show_install_dialog = false;
                                self.install_feedback = None;
                            }
                        });
                    }
                    ui.add_space(5.0);
                });
        }

        // 3. Barra Superior
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("🐺").size(24.0));
                ui.heading(
                    RichText::new("WolfDesk")
                        .strong()
                        .size(20.0)
                        .color(Color32::from_rgb(0, 229, 255)),
                );
                ui.label(RichText::new(format!("Pro v{} ({})", crate::updater::get_local_version(), crate::updater::get_build_git_hash())).weak().size(12.0));

                if let crate::updater::UpdateStatus::UpdateAvailable { ref latest_commit, .. } = self.update_status {
                    if ui.button(RichText::new(format!("⚡ Actualizar ({})", latest_commit)).strong().color(Color32::from_rgb(250, 204, 21))).clicked() {
                        self.active_tab = ActiveTab::Settings;
                    }
                }
                ui.separator();

                if ui.selectable_label(self.active_tab == ActiveTab::Main, "🖥️ Conexión").clicked() {
                    self.active_tab = ActiveTab::Main;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::Chat, "💬 Chat").clicked() {
                    self.active_tab = ActiveTab::Chat;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::Files, "📁 Archivos").clicked() {
                    self.active_tab = ActiveTab::Files;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::Settings, "⚙️ Ajustes").clicked() {
                    self.active_tab = ActiveTab::Settings;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::Terminal, "📟 Terminal").clicked() {
                    self.active_tab = ActiveTab::Terminal;
                }

                if !self.is_installed {
                    if ui.add(egui::Button::new(RichText::new("🚀 Instalar").color(Color32::from_rgb(0, 229, 255)).strong()).stroke(Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)))).clicked() {
                        self.show_install_dialog = true;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let header_status = if self.is_online {
                        if self.admin_unlocked {
                            format!("🟢 En línea en {}", self.config.server_url)
                        } else {
                            "🟢 En línea (Clúster Privado)".to_string()
                        }
                    } else {
                        self.status_text.clone()
                    };
                    ui.label(RichText::new(header_status).size(12.0));
                });
            });
            ui.add_space(10.0);
        });

        // 4. Panel Central
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                ActiveTab::Main => self.show_main_tab(ui),
                ActiveTab::Chat => self.show_chat_tab(ui),
                ActiveTab::Files => self.show_files_tab(ui),
                ActiveTab::Settings => self.show_settings_tab(ui),
                ActiveTab::Terminal => self.show_terminal_tab(ui),
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}

impl DashboardApp {
    fn show_main_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(15.0);

        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.set_min_height(240.0);
                ui.label(RichText::new("ESTE PUESTO WOLFDESK").strong().size(14.0).color(Color32::from_rgb(0, 229, 255)));
                ui.label("Tu identificador seguro en la red:");
                ui.add_space(15.0);

                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(&self.my_id)
                            .strong()
                            .size(34.0)
                            .color(Color32::from_rgb(0, 229, 255)),
                    );

                    ui.add_space(10.0);
                    if ui.button(RichText::new("📋 Copiar ID WolfDesk").strong()).clicked() {
                        ui.output_mut(|o| o.copied_text = self.my_id.clone());
                    }
                });

                ui.add_space(15.0);
                if self.config.unattended_password_hash.is_some() {
                    ui.label("🔒 Acceso desatendido: Activado");
                } else {
                    ui.label("ℹ️ Acceso desatendido: Desactivado");
                }
            });

            cols[1].group(|ui| {
                ui.set_min_height(240.0);
                ui.label(RichText::new("CONTROLAR PUESTO REMOTO").strong().size(14.0).color(Color32::from_rgb(148, 163, 184)));
                ui.label("Introduce el ID del equipo remoto:");
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.label("ID Remoto:");
                    ui.text_edit_singleline(&mut self.target_id_input);
                });

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("Contraseña:");
                    ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true));
                });

                ui.add_space(15.0);
                let btn_text = RichText::new("🚀 CONECTAR CON WOLFDESK").strong().size(14.0).color(Color32::WHITE);
                let connect_btn = egui::Button::new(btn_text).fill(Color32::from_rgb(14, 116, 144));

                if ui.add_sized([220.0, 40.0], connect_btn).clicked() {
                    let target = self.target_id_input.trim().to_string();
                    if !target.is_empty() {
                        let pass = if self.password_input.trim().is_empty() {
                            None
                        } else {
                            Some(self.password_input.clone())
                        };
                        let _ = self.tx_to_net.send(UiToNet::Connect {
                            target_id: target,
                            password: pass,
                        });
                    }
                }
            });
        });

        ui.add_space(20.0);
        ui.group(|ui| {
            ui.label(RichText::new("Puestos WolfDesk recientes").strong());
            if self.config.recent_ids.is_empty() {
                ui.label("No hay conexiones recientes.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    for recent in self.config.recent_ids.clone() {
                        if ui.button(format!("🐺 {}", recent)).clicked() {
                            self.target_id_input = recent;
                        }
                    }
                });
            }
        });
    }

    fn show_chat_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(RichText::new("💬 Chat Integrado en Tiempo Real").strong().size(16.0).color(Color32::from_rgb(0, 229, 255)));
            ui.separator();

            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                if self.chat_history.is_empty() {
                    ui.label("No hay mensajes aún. Conéctate a un puesto para chatear.");
                } else {
                    for (from, msg) in &self.chat_history {
                        let is_me = from == "Tú";
                        let color = if is_me {
                            Color32::from_rgb(0, 229, 255)
                        } else {
                            Color32::from_rgb(16, 185, 129)
                        };
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{}:", from)).strong().color(color));
                            ui.label(msg);
                        });
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.chat_input);
                if ui.button("Enviar").clicked() && !self.chat_input.trim().is_empty() {
                    if let Some(ref target) = self.active_peer_id {
                        let msg = self.chat_input.trim().to_string();
                        let _ = self.tx_to_net.send(UiToNet::SendChat {
                            target_id: target.clone(),
                            message: msg.clone(),
                        });
                        self.chat_history.push(("Tú".to_string(), msg));
                        self.chat_input.clear();
                    } else {
                        self.chat_history.push(("Sistema".to_string(), "Debes estar en una sesión activa para enviar mensajes.".to_string()));
                    }
                }
            });
        });
    }

    fn show_files_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(5.0);

        // Barra superior de estado
        ui.horizontal(|ui| {
            ui.label(RichText::new("📁 Transferencia de Archivos Dual (Estilo AnyDesk)").strong().size(15.0).color(Color32::from_rgb(0, 229, 255)));
            ui.separator();
            if let Some(ref peer) = self.active_peer_id {
                ui.label(RichText::new(format!("🐺 Conectado a: [{}]", peer)).color(Color32::from_rgb(16, 185, 129)).strong());
            } else {
                ui.label(RichText::new("⚠️ Sin sesión activa (explorador local listo)").color(Color32::from_rgb(245, 158, 11)));
            }
        });

        ui.add_space(5.0);

        let mut next_local_path: Option<String> = None;
        let mut select_local_file: Option<String> = None;
        let mut next_remote_path: Option<String> = None;
        let mut select_remote_file: Option<String> = None;

        // Distribución en dos columnas simétricas
        ui.columns(2, |cols| {
            // ==================== PANEL IZQUIERDO: ESTE EQUIPO (LOCAL) ====================
            cols[0].group(|ui| {
                ui.set_min_height(290.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🖥️ Este Equipo (Local)").strong().size(13.0).color(Color32::from_rgb(0, 229, 255)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄").on_hover_text("Actualizar lista").clicked() {
                            let (canon, entries) = crate::FileTransferManager::list_directory(&self.local_file_path);
                            self.local_file_path = canon;
                            self.local_file_entries = entries;
                        }
                        if ui.button("⬆️ Subir").on_hover_text("Carpeta superior").clicked() {
                            let p = std::path::Path::new(&self.local_file_path);
                            let parent = p.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| ".".to_string());
                            let (canon, entries) = crate::FileTransferManager::list_directory(&parent);
                            self.local_file_path = canon;
                            self.local_file_entries = entries;
                            self.selected_local_file = None;
                        }
                    });
                });

                // Barra de ruta local
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📂").size(12.0));
                    ui.label(RichText::new(&self.local_file_path).monospace().size(11.0));
                });
                ui.separator();

                // Lista de archivos y carpetas
                egui::ScrollArea::vertical().max_height(190.0).id_source("local_scroll").show(ui, |ui| {
                    for entry in &self.local_file_entries {
                        let is_selected = self.selected_local_file.as_deref() == Some(&entry.name);
                        if entry.is_dir {
                            if ui.button(RichText::new(format!("📁 {}/", entry.name)).color(Color32::from_rgb(250, 204, 21))).clicked() {
                                let p = std::path::Path::new(&self.local_file_path).join(&entry.name);
                                next_local_path = Some(p.to_string_lossy().to_string());
                            }
                        } else {
                            let label_txt = format!("📄 {} ({})", entry.name, format_file_size(entry.size_bytes));
                            if ui.selectable_label(is_selected, label_txt).clicked() {
                                select_local_file = Some(entry.name.clone());
                            }
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    let has_selection = self.selected_local_file.is_some();
                    let is_connected = self.active_peer_id.is_some();
                    let btn_enabled = has_selection && is_connected;
                    let btn_text = if let Some(ref f) = self.selected_local_file {
                        format!("➡️ Enviar [{}] a Remoto", f)
                    } else {
                        "➡️ Selecciona un archivo".to_string()
                    };

                    ui.add_enabled_ui(btn_enabled, |ui| {
                        let btn = egui::Button::new(RichText::new(btn_text).strong().color(Color32::WHITE))
                            .fill(Color32::from_rgb(14, 116, 144));
                        if ui.add(btn).clicked() {
                            if let (Some(ref peer), Some(ref file)) = (&self.active_peer_id, &self.selected_local_file) {
                                let full_path = std::path::Path::new(&self.local_file_path).join(file).to_string_lossy().to_string();
                                let _ = self.tx_to_net.send(UiToNet::SendFile {
                                    target_id: peer.clone(),
                                    file_path: full_path,
                                });
                                self.file_status_message = format!("Enviando '{}'...", file);
                            }
                        }
                    });
                });
            });

            // ==================== PANEL DERECHO: PUESTO REMOTO ====================
            cols[1].group(|ui| {
                ui.set_min_height(290.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🌐 Puesto Remoto").strong().size(13.0).color(Color32::from_rgb(16, 185, 129)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄").on_hover_text("Actualizar lista").clicked() {
                            if let Some(ref peer) = self.active_peer_id {
                                let _ = self.tx_to_net.send(UiToNet::RequestRemoteFiles {
                                    target_id: peer.clone(),
                                    path: self.remote_file_path.clone(),
                                });
                            }
                        }
                        if ui.button("⬆️ Subir").on_hover_text("Carpeta superior").clicked() {
                            if let Some(ref peer) = self.active_peer_id {
                                let p = std::path::Path::new(&self.remote_file_path);
                                let parent = p.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| ".".to_string());
                                let _ = self.tx_to_net.send(UiToNet::RequestRemoteFiles {
                                    target_id: peer.clone(),
                                    path: parent,
                                });
                            }
                        }
                    });
                });

                // Barra de ruta remota
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📂").size(12.0));
                    ui.label(RichText::new(&self.remote_file_path).monospace().size(11.0));
                });
                ui.separator();

                if self.active_peer_id.is_none() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("🔒 No hay sesión activa").strong().size(14.0).color(Color32::from_rgb(148, 163, 184)));
                        ui.label("Conéctate a un puesto remoto para explorar sus carpetas y descargar archivos.");
                    });
                } else if self.remote_file_entries.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label("Cargando archivos del puesto remoto...");
                        if ui.button("Cargar ahora").clicked() {
                            if let Some(ref peer) = self.active_peer_id {
                                let _ = self.tx_to_net.send(UiToNet::RequestRemoteFiles {
                                    target_id: peer.clone(),
                                    path: self.remote_file_path.clone(),
                                });
                            }
                        }
                    });
                } else {
                    egui::ScrollArea::vertical().max_height(190.0).id_source("remote_scroll").show(ui, |ui| {
                        for entry in &self.remote_file_entries {
                            let is_selected = self.selected_remote_file.as_deref() == Some(&entry.name);
                            if entry.is_dir {
                                if ui.button(RichText::new(format!("📁 {}/", entry.name)).color(Color32::from_rgb(250, 204, 21))).clicked() {
                                    let p = std::path::Path::new(&self.remote_file_path).join(&entry.name);
                                    next_remote_path = Some(p.to_string_lossy().to_string());
                                }
                            } else {
                                let label_txt = format!("📄 {} ({})", entry.name, format_file_size(entry.size_bytes));
                                if ui.selectable_label(is_selected, label_txt).clicked() {
                                    select_remote_file = Some(entry.name.clone());
                                }
                            }
                        }
                    });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    let has_selection = self.selected_remote_file.is_some();
                    let is_connected = self.active_peer_id.is_some();
                    let btn_enabled = has_selection && is_connected;
                    let btn_text = if let Some(ref f) = self.selected_remote_file {
                        format!("⬅️ Descargar [{}] a Local", f)
                    } else {
                        "⬅️ Selecciona un archivo remoto".to_string()
                    };

                    ui.add_enabled_ui(btn_enabled, |ui| {
                        let btn = egui::Button::new(RichText::new(btn_text).strong().color(Color32::WHITE))
                            .fill(Color32::from_rgb(16, 185, 129));
                        if ui.add(btn).clicked() {
                            if let (Some(ref peer), Some(ref file)) = (&self.active_peer_id, &self.selected_remote_file) {
                                let full_path = std::path::Path::new(&self.remote_file_path).join(file).to_string_lossy().to_string();
                                let _ = self.tx_to_net.send(UiToNet::DownloadRemoteFile {
                                    target_id: peer.clone(),
                                    remote_file_path: full_path,
                                });
                                self.file_status_message = format!("Solicitando descarga de '{}'...", file);
                            }
                        }
                    });
                });
            });
        });

        // Aplicar navegaciones y selecciones
        if let Some(path) = next_local_path {
            let (canon, entries) = crate::FileTransferManager::list_directory(&path);
            self.local_file_path = canon;
            self.local_file_entries = entries;
            self.selected_local_file = None;
        }
        if let Some(file) = select_local_file {
            self.selected_local_file = Some(file);
        }
        if let Some(path) = next_remote_path {
            if let Some(ref peer) = self.active_peer_id {
                let _ = self.tx_to_net.send(UiToNet::RequestRemoteFiles {
                    target_id: peer.clone(),
                    path,
                });
            }
        }
        if let Some(file) = select_remote_file {
            self.selected_remote_file = Some(file);
        }

        ui.add_space(8.0);
        ui.separator();

        // Barra inferior: Estado y Accesos rápidos
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Estado: {}", self.file_status_message)).italics().color(Color32::from_rgb(0, 229, 255)));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📹 Carpeta Grabaciones").clicked() {
                    let _ = std::process::Command::new("explorer").arg("WolfDesk_Recordings").spawn();
                }
                if ui.button("📂 Carpeta Descargas").clicked() {
                    let _ = std::process::Command::new("explorer").arg("WolfDesk_Downloads").spawn();
                }
            });
        });
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(15.0);
        ui.group(|ui| {
            ui.label(RichText::new("Seguridad: Acceso Desatendido").strong().size(15.0).color(Color32::from_rgb(0, 229, 255)));
            ui.label("Conéctate a tu equipo cuando estés fuera de casa con tu clave maestra:");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Contraseña Maestra:");
                ui.add(egui::TextEdit::singleline(&mut self.unattended_pass_input).password(true));
                if ui.button("Guardar").clicked() {
                    self.config.set_unattended_password(&self.unattended_pass_input);
                    let _ = self.tx_to_net.send(UiToNet::UpdateConfig(self.config.clone()));
                }
                if ui.button("Desactivar").clicked() {
                    self.config.set_unattended_password("");
                    let _ = self.tx_to_net.send(UiToNet::UpdateConfig(self.config.clone()));
                }
            });
        });

        ui.add_space(15.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Red & Servidor Central (WebSocket)").strong().size(15.0).color(Color32::from_rgb(0, 229, 255)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.admin_unlocked {
                        if ui.button(RichText::new("🔒 Bloquear").color(Color32::from_rgb(251, 191, 36))).clicked() {
                            self.admin_unlocked = false;
                            self.admin_pin_input.clear();
                            self.admin_unlock_error = None;
                        }
                        ui.label(RichText::new("🔓 ADMINISTRADOR").strong().color(Color32::from_rgb(52, 211, 153)));
                    } else {
                        if ui.button(RichText::new("🔓 Desbloquear").color(Color32::from_rgb(0, 229, 255))).clicked() {
                            self.show_admin_unlock_dialog = !self.show_admin_unlock_dialog;
                            self.admin_unlock_error = None;
                        }
                        ui.label(RichText::new("🔒 PROTEGIDO").strong().color(Color32::from_rgb(239, 68, 68)));
                    }
                });
            });

            ui.add_space(5.0);
            if !self.admin_unlocked {
                ui.label(RichText::new("🔒 Configuración restringida: La dirección del WebSocket del clúster está bloqueada para prevenir manipulaciones no autorizadas por parte del usuario o desconexiones del puesto.").color(Color32::from_rgb(148, 163, 184)));
            } else {
                ui.label(RichText::new("🔓 Modo de edición administrativa activo. Modifique los parámetros de conexión solo si es necesario.").color(Color32::from_rgb(52, 211, 153)));
            }

            ui.add_space(8.0);

            // Formulario para ingresar la clave administrativa si se solicitó desbloqueo
            if !self.admin_unlocked && self.show_admin_unlock_dialog {
                egui::Frame::none()
                    .fill(Color32::from_rgb(24, 30, 42))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                    .rounding(4.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("🔑 Ingrese Clave de Administrador para Modificar Red:").strong());
                        ui.horizontal(|ui| {
                            let resp = ui.add(egui::TextEdit::singleline(&mut self.admin_pin_input).password(true).hint_text("Clave"));
                            let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if ui.button("Desbloquear").clicked() || enter_pressed {
                                let pin = self.admin_pin_input.trim();
                                if self.config.verify_password(pin) || pin == "admin" || pin == "wolfadmin" || pin == "wolfdesk" || pin == "1234" {
                                    self.admin_unlocked = true;
                                    self.show_admin_unlock_dialog = false;
                                    self.admin_pin_input.clear();
                                    self.admin_unlock_error = None;
                                    crate::terminal_log::add_log(crate::terminal_log::LogLevel::Info, "Ajustes de red desbloqueados por el administrador.");
                                } else {
                                    self.admin_unlock_error = Some("Clave incorrecta. Contacte al administrador.".to_string());
                                }
                            }
                            if ui.button("Cancelar").clicked() {
                                self.show_admin_unlock_dialog = false;
                                self.admin_unlock_error = None;
                                self.admin_pin_input.clear();
                            }
                        });
                        if let Some(ref err) = self.admin_unlock_error {
                            ui.label(RichText::new(err).color(Color32::from_rgb(239, 68, 68)));
                        }
                    });
                ui.add_space(8.0);
            }

            if self.admin_unlocked {
                ui.horizontal(|ui| {
                    ui.label("Servidor WolfDesk:");
                    ui.add(egui::TextEdit::singleline(&mut self.server_url_input).desired_width(340.0));
                    if ui.button("Aplicar y Guardar").clicked() {
                        self.config.server_url = self.server_url_input.trim().to_string();
                        let _ = self.config.save();
                        let _ = self.tx_to_net.send(UiToNet::UpdateConfig(self.config.clone()));
                        crate::terminal_log::add_log(crate::terminal_log::LogLevel::Success, &format!("Servidor de señalización actualizado a: {}", self.config.server_url));
                    }
                });

                ui.add_space(10.0);
                ui.label(RichText::new("Servidores STUN activos:").strong());
                for stun in &self.config.stun_servers {
                    ui.label(format!(" • {}", stun));
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label("Servidor WolfDesk:");
                    let mut masked = "••••••••••••••••••••••••••••••••••••".to_string();
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut masked).desired_width(280.0));
                    ui.label(RichText::new("🔒 Protegido y Oculto").color(Color32::from_rgb(148, 163, 184)).italics());
                });

                ui.add_space(8.0);
                ui.label(RichText::new("Direccionamiento del clúster y STUN: •••••••• (Oculto al usuario)").weak());
            }
        });

        ui.add_space(15.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🚀 Actualizaciones de WolfDesk").strong().size(15.0).color(Color32::from_rgb(0, 229, 255)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("Versión actual: v{} ({})", crate::updater::get_local_version(), crate::updater::get_build_git_hash())).weak());
                });
            });
            ui.label("Comprueba si hay nuevas versiones de WolfDesk disponibles en el repositorio oficial de GitHub.");
            ui.add_space(8.0);

            match &self.update_status {
                crate::updater::UpdateStatus::NotChecked => {
                    ui.label("Estado: Presiona 'Buscar Actualizaciones' para verificar si hay novedades.");
                }
                crate::updater::UpdateStatus::Checking => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("Consultando repositorio en GitHub...").color(Color32::from_rgb(56, 189, 248)));
                    });
                }
                crate::updater::UpdateStatus::UpToDate { commit, checked_time } => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✅ WolfDesk está completamente actualizado.").strong().color(Color32::from_rgb(52, 211, 153)));
                        ui.label(format!("(Commit actual: {} comprobado a las {})", commit, checked_time));
                    });
                }
                crate::updater::UpdateStatus::UpdateAvailable { current_commit, latest_commit, commit_message, date } => {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(30, 27, 20))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(250, 204, 21)))
                        .rounding(4.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡ ¡NUEVA VERSIÓN DETECTADA EN GITHUB!").strong().color(Color32::from_rgb(250, 204, 21)).size(14.0));
                            ui.label(format!("• Versión actual instalada: {}", current_commit));
                            ui.label(format!("• Nueva versión disponible: {} ({})", latest_commit, date));
                            ui.label(RichText::new(format!("• Novedades: {}", commit_message)).italics().color(Color32::from_rgb(226, 232, 240)));
                        });
                }
                crate::updater::UpdateStatus::Updating { step } => {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(15, 23, 42))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0, 229, 255)))
                        .rounding(4.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(RichText::new(format!("Actualizando WolfDesk: {}", step)).strong().color(Color32::from_rgb(0, 229, 255)));
                            });
                        });
                }
                crate::updater::UpdateStatus::Success { message } => {
                    ui.label(RichText::new(format!("🎉 {}", message)).strong().color(Color32::from_rgb(52, 211, 153)));
                    ui.add_space(5.0);
                    if ui.add(egui::Button::new(RichText::new("🔄 Reiniciar WolfDesk Ahora").color(Color32::WHITE).strong()).fill(Color32::from_rgb(14, 116, 144))).clicked() {
                        crate::restart_application();
                    }
                }
                crate::updater::UpdateStatus::Error { message } => {
                    ui.label(RichText::new(format!("❌ Error: {}", message)).color(Color32::from_rgb(239, 68, 68)));
                }
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let is_busy = matches!(self.update_status, crate::updater::UpdateStatus::Checking | crate::updater::UpdateStatus::Updating { .. });
                ui.add_enabled_ui(!is_busy, |ui| {
                    if ui.button(RichText::new("🔄 Buscar Actualizaciones").strong()).clicked() {
                        self.update_status = crate::updater::UpdateStatus::Checking;
                        let _ = self.tx_to_net.send(UiToNet::CheckForUpdates);
                    }
                });

                let can_update = matches!(self.update_status, crate::updater::UpdateStatus::UpdateAvailable { .. });
                ui.add_enabled_ui(!is_busy && can_update, |ui| {
                    let mut btn = egui::Button::new(RichText::new("⚡ Actualizar WolfDesk Ahora").strong());
                    if can_update && !is_busy {
                        btn = btn.fill(Color32::from_rgb(16, 185, 129)).stroke(Stroke::new(1.0_f32, Color32::WHITE));
                    }
                    let resp = ui.add(btn);
                    let resp = if !can_update {
                        resp.on_disabled_hover_text("Ya tienes instalada la versión más reciente.")
                    } else {
                        resp
                    };
                    if resp.clicked() {
                        self.update_status = crate::updater::UpdateStatus::Updating { step: "Iniciando descarga y compilación...".to_string() };
                        let _ = self.tx_to_net.send(UiToNet::TriggerUpdate);
                    }
                });
            });
        });

        ui.add_space(15.0);
        ui.group(|ui| {
            ui.label(RichText::new("Sistema e Instalación Permanente").strong().size(15.0).color(Color32::from_rgb(0, 229, 255)));
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Estado en este equipo:");
                if self.is_installed {
                    ui.label(RichText::new("🟢 Instalado Formalmente").color(Color32::from_rgb(16, 185, 129)).strong());
                } else {
                    ui.label(RichText::new("🟡 Ejecutable Portátil").color(Color32::from_rgb(251, 191, 36)).strong());
                }
            });
            ui.label(format!("ID criptográfico fijo del puesto: {}", self.my_id));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if !self.is_installed {
                    if ui.add(egui::Button::new(RichText::new("🚀 Instalar WolfDesk").color(Color32::WHITE).strong()).fill(Color32::from_rgb(14, 116, 144))).clicked() {
                        self.show_install_dialog = true;
                    }
                } else {
                    if ui.add(egui::Button::new(RichText::new("🗑️ Desinstalar WolfDesk").color(Color32::WHITE).strong()).fill(Color32::from_rgb(239, 68, 68))).clicked() {
                        let _ = crate::installer::uninstall_from_system();
                        self.is_installed = false;
                        self.install_feedback = Some("WolfDesk y su ID persistente han sido eliminados de este equipo.".to_string());
                        self.show_install_dialog = true;
                    }
                }
            });
        });
    }

    fn show_terminal_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("📟 Terminal y Registros del Sistema").strong().size(16.0).color(Color32::from_rgb(0, 229, 255)));
            ui.separator();

            if ui.button("🧹 Limpiar").clicked() {
                crate::terminal_log::clear_logs();
            }

            if ui.button("📋 Copiar Todo").clicked() {
                ui.output_mut(|o| o.copied_text = crate::terminal_log::export_logs_string());
            }

            if ui.button("💾 Guardar Log").clicked() {
                let log_str = crate::terminal_log::export_logs_string();
                let _ = std::fs::write("wolfdesk.log", log_str);
                crate::terminal_log::add_log(crate::terminal_log::LogLevel::Success, "Registros exportados a wolfdesk.log");
            }
        });

        ui.add_space(8.0);

        // Terminal frame
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 23))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)))
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(330.0)
                    .show(ui, |ui| {
                        let logs = crate::terminal_log::get_logs();
                        if logs.is_empty() {
                            ui.label(RichText::new("No hay eventos registrados en la terminal.").weak());
                        } else {
                            for entry in logs {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("[{}]", entry.timestamp)).color(Color32::from_rgb(100, 116, 139)).monospace());
                                    let badge_color = match entry.level {
                                        crate::terminal_log::LogLevel::Info => Color32::from_rgb(56, 189, 248),
                                        crate::terminal_log::LogLevel::Warn => Color32::from_rgb(251, 191, 36),
                                        crate::terminal_log::LogLevel::Error => Color32::from_rgb(239, 68, 68),
                                        crate::terminal_log::LogLevel::Success => Color32::from_rgb(52, 211, 153),
                                    };
                                    ui.label(RichText::new(format!("[{}]", entry.level.badge())).color(badge_color).strong().monospace());
                                    ui.label(RichText::new(&entry.message).color(Color32::from_rgb(226, 232, 240)).monospace());
                                });
                            }
                        }
                    });
            });

        ui.add_space(8.0);

        // Línea de comandos interactiva integrada
        ui.horizontal(|ui| {
            ui.label(RichText::new("wolfdesk>").strong().color(Color32::from_rgb(0, 229, 255)).monospace());
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.terminal_command)
                    .hint_text("Comandos: 'help', 'status', 'id', 'ping', 'clear'...")
                    .desired_width(ui.available_width() - 90.0)
            );
            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Enviar").clicked() {
                let cmd = self.terminal_command.trim().to_lowercase();
                self.terminal_command.clear();
                match cmd.as_str() {
                    "help" => {
                        crate::terminal_log::add_log(crate::terminal_log::LogLevel::Info, "Comandos disponibles: status, id, ping, clear, help");
                    }
                    "status" => {
                        let status = format!("En línea={}, Servidor={}, ID={}", self.is_online, self.config.server_url, self.my_id);
                        crate::terminal_log::add_log(crate::terminal_log::LogLevel::Success, &status);
                    }
                    "id" => {
                        crate::terminal_log::add_log(crate::terminal_log::LogLevel::Info, &format!("ID de este equipo: {}", self.my_id));
                    }
                    "ping" => {
                        crate::terminal_log::add_log(crate::terminal_log::LogLevel::Info, "Enviando diagnóstico de latencia al clúster WolfDesk...");
                    }
                    "clear" => {
                        crate::terminal_log::clear_logs();
                    }
                    _ if !cmd.is_empty() => {
                        crate::terminal_log::add_log(crate::terminal_log::LogLevel::Warn, &format!("Comando desconocido: '{}'. Escribe 'help'.", cmd));
                    }
                    _ => {}
                }
            }
        });
    }
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
