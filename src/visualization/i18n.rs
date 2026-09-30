//! Presentation-only translations. Stored/CLI diagnostics and scientific values stay unchanged.
use super::i18n_catalog::CATALOG;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    English,
    Korean,
}

impl Language {
    /// English strings are stable catalog keys. Format placeholders capture already
    /// formatted values; translation never rounds numbers or edits paths/identifiers.
    pub fn text<'a>(self, text: &'a str) -> Cow<'a, str> {
        if self == Self::English || text.is_empty() {
            return Cow::Borrowed(text);
        }
        if let Some((_, korean)) = CATALOG.iter().find(|(english, _)| *english == text) {
            return Cow::Borrowed(korean);
        }
        if let Some(detail) = text.strip_prefix("Auto-save FAILED: ") {
            for suffix in [
                "; calculated result retained",
                "; legacy calculation retained",
            ] {
                if let Some(error) = detail.strip_suffix(suffix) {
                    return Cow::Owned(format!(
                        "{}{}{}",
                        self.text("Auto-save FAILED: "),
                        self.diagnostic(error),
                        self.text(suffix)
                    ));
                }
            }
        }
        // These wrappers contain diagnostics, not opaque physical values or paths.
        for prefix in [
            "ERROR: ",
            "Load ERROR: ",
            "Export failed: ",
            "Detector display: ",
            "UI settings error: ",
            "OS seed generation failed: ",
            "cannot spawn ray worker: ",
        ] {
            if let Some(detail) = text.strip_prefix(prefix) {
                return Cow::Owned(format!("{}{}", self.text(prefix), self.diagnostic(detail)));
            }
        }
        for &(english, korean) in CATALOG {
            if english.contains('{')
                && let Some(values) = captures(english, text)
            {
                return Cow::Owned(interpolate(korean, &values));
            }
        }
        if text.contains('\n') {
            return Cow::Owned(
                text.split('\n')
                    .map(|line| self.text(line))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        Cow::Borrowed(text)
    }

    /// OS/library errors can be arbitrary and must remain intact for diagnosis.
    /// Known application errors are translated; external diagnostics are identified.
    pub fn diagnostic<'a>(self, text: &'a str) -> Cow<'a, str> {
        let translated = self.text(text);
        if self == Self::Korean && !text.is_empty() && translated.as_ref() == text {
            Cow::Owned(format!("원본 진단: {text}"))
        } else {
            translated
        }
    }
}

fn captures<'a>(template: &str, mut text: &'a str) -> Option<Vec<&'a str>> {
    let mut template = template;
    let mut values = Vec::new();
    while let Some(start) = template.find('{') {
        text = text.strip_prefix(&template[..start])?;
        let end = template[start..].find('}')? + start;
        template = &template[end + 1..];
        let literal_end = template.find('{').unwrap_or(template.len());
        let delimiter = &template[..literal_end];
        if delimiter.is_empty() {
            if !template.is_empty() {
                return None;
            }
            values.push(text);
            return Some(values);
        }
        let split = if literal_end == template.len() {
            text.strip_suffix(delimiter)?.len()
        } else {
            text.find(delimiter)?
        };
        values.push(&text[..split]);
        text = &text[split..];
    }
    (text == template).then_some(values)
}

fn interpolate(template: &str, values: &[&str]) -> String {
    let mut output = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        output.push_str(&rest[..start]);
        let end = rest[start..].find('}').expect("catalog placeholder") + start;
        let index: usize = rest[start + 1..end].parse().expect("catalog value index");
        output.push_str(values[index]);
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    output
}

/// Stable egui identity preserves expanded/collapsed state across language changes.
pub fn collapsing<R>(
    ui: &mut egui::Ui,
    language: Language,
    key: &'static str,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::CollapsingResponse<R> {
    header(ui, language, key).show(ui, body)
}

pub fn header(ui: &egui::Ui, language: Language, key: &'static str) -> egui::CollapsingHeader {
    // egui's CollapsingHeader forces Extend even when the surrounding style wraps.
    // Lay out the title explicitly so translated headers fit the resizable panel.
    let width =
        (ui.available_width() - ui.spacing().indent - 2. * ui.spacing().button_padding.x).max(1.);
    let text = egui::WidgetText::from(language.text(key)).into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        width,
        egui::TextStyle::Button,
    );
    egui::CollapsingHeader::new(text).id_salt(key)
}

pub fn numeric(ui: &mut egui::Ui, language: Language, widget: impl egui::Widget) -> egui::Response {
    ui.add(widget).on_hover_text(language.text(
        "Drag to edit or click to enter a value.\nPress Shift while dragging for better control.",
    ))
}

/// System-owned fonts are read in place, never bundled or redistributed.
/// App labels and monospace diagnostic text both need the Hangul fallback.
pub fn install_korean_font(ctx: &egui::Context) -> bool {
    let mut candidates = Vec::new();
    if let Some(windows) = std::env::var_os("WINDIR") {
        candidates.push(std::path::PathBuf::from(windows).join("Fonts/malgun.ttf"));
    }
    candidates.extend([
        "/System/Library/Fonts/AppleSDGothicNeo.ttc".into(),
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc".into(),
        "/usr/share/fonts/truetype/nanum/NanumGothic.ttf".into(),
    ]);
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            let name = "system-korean".to_owned();
            fonts
                .font_data
                .insert(name.clone(), egui::FontData::from_owned(bytes).into());
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push(name.clone());
            }
            ctx.set_fonts(fonts);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_unique_keys_and_valid_placeholders() {
        let mut keys = std::collections::HashSet::new();
        for &(en, ko) in CATALOG {
            assert!(keys.insert(en), "duplicate key {en}");
            assert!(!ko.is_empty());
            if en.contains('{') {
                let mut example = String::new();
                let mut rest = en;
                let mut count = 0;
                while let Some(start) = rest.find('{') {
                    example.push_str(&rest[..start]);
                    example.push_str(&format!("VALUE_{count}"));
                    count += 1;
                    rest = &rest[rest.find('}').unwrap() + 1..];
                }
                example.push_str(rest);
                let values = captures(en, &example).unwrap_or_else(|| panic!("pattern {en}"));
                let translated = interpolate(ko, &values);
                for i in 0..count {
                    assert!(
                        translated.contains(&format!("VALUE_{i}")),
                        "value lost: {en}"
                    );
                }
            }
        }
    }

    #[test]
    fn translation_preserves_opaque_paths_values_and_internal_status_codes() {
        let path = r"C:\research\Ray {1}\DET\simulation_1";
        assert_eq!(
            Language::Korean.text(&format!("Loaded folder: {path}")),
            format!("불러온 폴더: {path}")
        );
        for token in [
            "chi",
            "u_hit",
            "v_hit",
            "t_hit",
            "E",
            "L_z",
            "Q",
            "DET",
            "CAP",
            "ESC",
            "NUM",
            "ACT",
            "--input",
            "sim_1.bin",
        ] {
            assert_eq!(Language::Korean.text(token), token);
        }
        assert_eq!(
            Language::Korean.text("Rendered trajectories 512"),
            "표시 궤적 512"
        );
        assert_eq!(
            Language::English.text("Rendered trajectories 512"),
            "Rendered trajectories 512"
        );
    }
}
