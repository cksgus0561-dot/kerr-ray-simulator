//! Presentation regression tests: no geodesic calculation or archive rewrite.
use eframe::egui;
use kerr_ray::{
    session::SessionConfig,
    visualization::{
        i18n::{self, Language},
        preferences::UiPreferences,
        ui,
    },
};

#[test]
fn language_sidecar_roundtrips_without_simulation_settings() {
    let path = std::env::temp_dir().join(format!("kerr-ui-language-{}.json", std::process::id()));
    for language in [Language::Korean, Language::English, Language::Korean] {
        UiPreferences { language }.save(&path).unwrap();
        assert_eq!(UiPreferences::load(&path).unwrap().language, language);
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(json.as_object().unwrap().len(), 1);
        assert!(json.get("language").is_some());
    }
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        UiPreferences::load(&path).unwrap().language,
        Language::English
    );
}

#[test]
fn old_or_invalid_ui_preferences_do_not_silently_change_physics_settings() {
    let default: UiPreferences = serde_json::from_str("{}").unwrap();
    assert_eq!(default.language, Language::English);
    assert!(serde_json::from_str::<UiPreferences>(r#"{"language":"unsupported"}"#).is_err());
    let cfg = SessionConfig::default();
    let json = serde_json::to_value(&cfg).unwrap();
    assert!(json.get("language").is_none());
    assert!(json["view"].get("language").is_none());
}

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    i18n::install_korean_font(&ctx);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.wrap_mode = Some(egui::TextWrapMode::Wrap);
        style.explanation_tooltips = false;
        style.animation_time = 0.;
    });
    ctx.set_theme(egui::Theme::Dark);
    ctx
}

#[test]
fn switching_ui_language_preserves_complete_session_and_physics_key() {
    let ctx = context();
    let mut cfg = SessionConfig::default();
    let before = serde_json::to_vec(&cfg).unwrap();
    let key = cfg.physics_key();
    for language in [Language::Korean, Language::English, Language::Korean] {
        let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
            egui::CentralPanel::default().show(root, |ui| {
                ui::settings(ui, &mut cfg, language);
            });
        });
        output.textures_delta.clear();
        assert_eq!(serde_json::to_vec(&cfg).unwrap(), before);
        assert_eq!(cfg.physics_key(), key);
    }
}

#[test]
fn collapsing_header_keeps_identity_when_language_changes() {
    let ctx = context();
    let mut ids = Vec::new();
    for language in [Language::English, Language::Korean] {
        let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
            ids.push(
                i18n::collapsing(root, language, "Visibility", |_| {})
                    .header_response
                    .id,
            );
        });
        output.textures_delta.clear();
    }
    assert_eq!(ids.first(), ids.last());
}

#[test]
fn settings_fit_narrow_panels_in_both_languages() {
    let ctx = context();
    ctx.memory_mut(|memory| memory.set_everything_is_visible(true));
    for language in [Language::English, Language::Korean] {
        for width in [240.0, 310.0] {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 3000.),
                )),
                ..Default::default()
            };
            let mut measured = (0.0_f32, 0.0_f32);
            let mut output = ctx.run_ui(input, |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    let available = ui.available_width();
                    ui::settings(ui, &mut SessionConfig::default(), language);
                    measured = (ui.min_rect().width(), available);
                });
            });
            output.textures_delta.clear();
            if measured.0 > measured.1 + 1. {
                for shape in &output.shapes {
                    if let egui::Shape::Text(text) = &shape.shape
                        && text.pos.x + text.galley.size().x > width - 8.
                    {
                        eprintln!(
                            "wide text {:?}: {}",
                            text.galley.size(),
                            text.galley.job.text
                        );
                    }
                }
            }
            assert!(
                measured.0 <= measured.1 + 1.,
                "{language:?}: used {} > available {}",
                measured.0,
                measured.1
            );
        }
    }
}

#[test]
fn statuses_and_diagnostics_switch_without_changing_the_original_message() {
    let status = "ERROR: accepted state exceeds energy-scaled null tolerance".to_owned();
    let original = status.clone();
    assert!(
        Language::Korean
            .text(&status)
            .starts_with("오류: 수락된 상태")
    );
    assert_eq!(Language::English.text(&status), original);
    assert_eq!(status, original);
    assert_eq!(
        Language::Korean.diagnostic("external-library-code-42"),
        "원본 진단: external-library-code-42"
    );
    assert_eq!(
        Language::Korean.text("HorizonCutoff"),
        "HorizonCutoff (지평선 수치 종료)"
    );
}
