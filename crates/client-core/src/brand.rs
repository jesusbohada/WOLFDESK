use eframe::egui::Color32;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrandType {
    WolfDesk,
    FarmaDesk,
}

#[derive(Debug, Clone)]
pub struct BrandConfig {
    pub brand_type: BrandType,
    pub brand_id: &'static str,
    pub config_file_name: &'static str,
    pub default_server_url: &'static str,
    pub app_name: &'static str,
    pub window_title: &'static str,
    pub brand_title: &'static str,
    pub icon_symbol: &'static str,
    pub exe_name: &'static str,
    pub unix_bin_name: &'static str,
    pub github_owner_repo: &'static str,
    pub single_instance_port: u16,
    pub install_folder_name: &'static str,
    pub icon_asset_name: &'static str,

    // Paleta de colores oficial de la marca
    pub primary_accent: Color32,       // Color principal de acento/destacados
    pub secondary_accent: Color32,     // Color corporativo base
    pub button_primary: Color32,       // Fondo de botones de acción
    pub button_border: Color32,        // Borde destacado de botones
    pub header_text_color: Color32,    // Color del texto de marca en cabecera
    pub card_border: Color32,          // Borde de tarjetas
}

static CURRENT_BRAND: OnceLock<BrandConfig> = OnceLock::new();

/// Retorna la configuración de marca activa (WolfDesk o FarmaDesk)
pub fn get_brand() -> &'static BrandConfig {
    CURRENT_BRAND.get_or_init(|| {
        #[cfg(feature = "farmadesk")]
        {
            return BrandConfig::farmadesk();
        }

        #[cfg(not(feature = "farmadesk"))]
        {
            // Detección automática por nombre del ejecutable o variable de entorno
            if let Ok(exe) = std::env::current_exe() {
                if let Some(file_name) = exe.file_name().and_then(|n| n.to_str()) {
                    let lower = file_name.to_lowercase();
                    if lower.contains("farmadesk") || lower.contains("farmanorte") {
                        return BrandConfig::farmadesk();
                    }
                }
            }
            if let Ok(brand_var) = std::env::var("WOLFDESK_BRAND") {
                if brand_var.to_lowercase() == "farmadesk" {
                    return BrandConfig::farmadesk();
                }
            }
            BrandConfig::wolfdesk()
        }
    })
}

impl BrandConfig {
    /// Identidad y paleta oficial de WolfDesk
    pub fn wolfdesk() -> Self {
        Self {
            brand_type: BrandType::WolfDesk,
            brand_id: "wolfdesk",
            config_file_name: "remote_desktop_config.json",
            default_server_url: "ws://127.0.0.1:9050/ws",
            app_name: "WolfDesk Pro",
            window_title: "WolfDesk Pro - Escritorio Remoto",
            brand_title: "🐺 WolfDesk",
            icon_symbol: "🐺",
            exe_name: "wolfdesk.exe",
            unix_bin_name: "wolfdesk",
            github_owner_repo: "jesusbohada/WOLFDESK",
            single_instance_port: 19059,
            install_folder_name: "WolfDesk",
            icon_asset_name: "wolfdesk.png",

            primary_accent: Color32::from_rgb(0, 229, 255),    // Cyan Neón (#00E5FF)
            secondary_accent: Color32::from_rgb(16, 185, 129), // Esmeralda (#10B981)
            button_primary: Color32::from_rgb(14, 116, 144),   // Azul profundo (#0E7490)
            button_border: Color32::from_rgb(56, 189, 248),    // Borde celeste (#38BDF8)
            header_text_color: Color32::from_rgb(0, 229, 255),
            card_border: Color32::from_rgb(30, 41, 59),
        }
    }

    /// Identidad y paleta oficial de FarmaDesk (Farmanorte)
    /// Basada en el manual de identidad corporativa:
    /// - Azul Corporativo: #003399 (RGB: 0, 51, 153)
    /// - Cyan Brillante:   #00AFEF (RGB: 0, 175, 239)
    pub fn farmadesk() -> Self {
        Self {
            brand_type: BrandType::FarmaDesk,
            brand_id: "farmadesk",
            config_file_name: "farmadesk_config.json",
            default_server_url: "ws://127.0.0.1:9050/ws",
            app_name: "FarmaDesk Pro",
            window_title: "FarmaDesk Pro - Escritorio Remoto",
            brand_title: "🔷 FarmaDesk",
            icon_symbol: "🔷",
            exe_name: "farmadesk.exe",
            unix_bin_name: "farmadesk",
            github_owner_repo: "jesusbohada/FARMADESK",
            single_instance_port: 19060, // Puerto independiente para no interferir con WolfDesk
            install_folder_name: "FarmaDesk",
            icon_asset_name: "farmadesk.png",

            primary_accent: Color32::from_rgb(0, 175, 239),     // Cyan oficial (#00AFEF)
            secondary_accent: Color32::from_rgb(0, 51, 153),    // Azul corporativo (#003399)
            button_primary: Color32::from_rgb(0, 51, 153),      // Fondo azul Farmanorte (#003399)
            button_border: Color32::from_rgb(0, 175, 239),      // Borde cyan (#00AFEF)
            header_text_color: Color32::from_rgb(0, 175, 239), // Encabezado en Cyan brillante
            card_border: Color32::from_rgb(15, 35, 75),         // Borde en azul profundo
        }
    }

    pub fn is_farmadesk(&self) -> bool {
        self.brand_type == BrandType::FarmaDesk
    }

    pub fn is_wolfdesk(&self) -> bool {
        self.brand_type == BrandType::WolfDesk
    }
}
