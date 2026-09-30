//! Small viewer-only sidecar, following the existing beside-executable JSON convention.
//! It never enters SessionConfig, physics keys, archives, hashes or worker requests.
use super::i18n::Language;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPreferences {
    pub language: Language,
}
impl UiPreferences {
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, bytes).map_err(|e| e.to_string())
    }
}

pub struct PreferencesUi {
    pub value: UiPreferences,
    path: Option<PathBuf>,
    error: Option<String>,
    korean_font: bool,
}
impl PreferencesUi {
    pub fn new(korean_font: bool) -> Self {
        let path = std::env::current_exe()
            .ok()
            .map(|p| p.with_file_name("kerr-viewer.ui.json"));
        let result = path
            .as_ref()
            .ok_or_else(|| "Cannot locate UI preferences beside executable".to_owned())
            .and_then(|p| UiPreferences::load(p));
        let (mut value, error) = match result {
            Ok(value) => (value, None),
            Err(error) => (UiPreferences::default(), Some(error)),
        };
        if !korean_font {
            value.language = Language::English;
        }
        Self {
            value,
            path,
            error,
            korean_font,
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let lang = self.value.language;
        ui.heading(lang.text("Settings"));
        let before = self.value.language;
        ui.horizontal_wrapped(|ui| {
            ui.label(lang.text("Language"));
            egui::ComboBox::from_id_salt("ui-language")
                .selected_text(match self.value.language {
                    Language::English => "English",
                    Language::Korean => "한국어",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.value.language, Language::English, "English");
                    ui.add_enabled_ui(self.korean_font, |ui| {
                        ui.selectable_value(&mut self.value.language, Language::Korean, "한국어");
                    });
                });
        });
        if before != self.value.language {
            self.error = self.path.as_ref().map_or_else(
                || Some("Cannot locate UI preferences beside executable".to_owned()),
                |p| self.value.save(p).err(),
            );
            ui.ctx().request_repaint();
        }
        if !self.korean_font {
            ui.colored_label(egui::Color32::YELLOW, lang.text("Korean system font unavailable. Install a Hangul-capable system font and restart."));
        }
        if let Some(error) = &self.error {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                self.value
                    .language
                    .text(&format!("UI settings error: {error}")),
            );
        }
        ui.separator();
    }
}
