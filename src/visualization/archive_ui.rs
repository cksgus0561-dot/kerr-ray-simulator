//! Folder browsing performs IO only. Reproduce is dispatched to the reference worker.
use super::i18n::{Language, collapsing, header};
use crate::standard_run::{self, Comparison};
use eframe::egui;
use std::{path::PathBuf, sync::mpsc};

type Loaded = Result<(PathBuf, Vec<String>, String, String, bool), String>;
pub struct ArchiveUi {
    pub auto_save_root: String,
    folder: String,
    loaded_folder: Option<PathBuf>,
    files: Vec<String>,
    selected: String,
    pending: Option<mpsc::Receiver<Loaded>>,
    valid: bool,
    message: String,
    pub save_message: String,
    pub comparison: Option<Comparison>,
}
impl Default for ArchiveUi {
    fn default() -> Self {
        Self {
            auto_save_root: "results/standard".into(),
            folder: "results/standard/simulation_1".into(),
            loaded_folder: None,
            files: vec![],
            selected: String::new(),
            pending: None,
            valid: false,
            message: String::new(),
            save_message: String::new(),
            comparison: None,
        }
    }
}
impl ArchiveUi {
    fn load(&mut self, directory: PathBuf, selection: Option<String>) {
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        self.valid = false;
        self.comparison = None;
        self.message = "Reading archive (no physics calculation)...".into();
        std::thread::spawn(move || {
            let result = (|| {
                let files =
                    standard_run::list_simulations(&directory).map_err(|e| e.to_string())?;
                let selected = selection.unwrap_or_else(|| files[0].clone());
                let (summary, valid) = match standard_run::load(&directory, &selected) {
                    Ok(run) => (
                        format!(
                            "{} rays / chi {}\nStored D/C/E/F: {:?}\nNo trajectories stored. Use Reproduce for fresh paths.",
                            run.rows.len(),
                            run.chi,
                            &standard_run::counts(&run.rows)[1..]
                        ),
                        true,
                    ),
                    Err(e) => (format!("Load ERROR: {e}"), false),
                };
                Ok((directory, files, selected, summary, valid))
            })();
            let _ = tx.send(result);
        });
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        busy: bool,
        lang: Language,
    ) -> Option<(PathBuf, String)> {
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    match result {
                        Ok((dir, files, selected, summary, valid)) => {
                            self.loaded_folder = Some(dir);
                            self.files = files;
                            self.selected = selected;
                            self.message = summary;
                            self.valid = valid;
                        }
                        Err(e) => self.message = format!("Load ERROR: {e}"),
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.message = "Load worker disconnected".into();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let mut request = None;
        header(ui, lang, "Standard simulation archive")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(lang.text("Auto-save root (new calculations only)"));
                ui.text_edit_singleline(&mut self.auto_save_root);
                ui.label(lang.diagnostic(&self.save_message));
                ui.label(lang.text("Simulation folder path"));
                ui.text_edit_singleline(&mut self.folder);
                if ui
                    .add_enabled(!busy, egui::Button::new(lang.text("Load Folder")))
                    .clicked()
                {
                    self.load(PathBuf::from(&self.folder), None);
                }
                if let Some(dir) = &self.loaded_folder {
                    ui.small(lang.text(&format!("Loaded folder: {}", dir.display())));
                }
                let old = self.selected.clone();
                ui.add_enabled_ui(!busy, |ui| {
                    egui::ComboBox::from_id_salt("standard-simulation-selection")
                        .selected_text(self.selected.trim_end_matches(".parquet"))
                        .show_ui(ui, |ui| {
                            for name in &self.files {
                                ui.selectable_value(
                                    &mut self.selected,
                                    name.clone(),
                                    name.trim_end_matches(".parquet"),
                                );
                            }
                        });
                });
                if old != self.selected
                    && let Some(dir) = self.loaded_folder.clone()
                {
                    self.load(dir, Some(self.selected.clone()));
                }
                ui.small(lang.diagnostic(&self.message));
                if ui
                    .add_enabled(
                        self.valid && self.pending.is_none() && !busy,
                        egui::Button::new(lang.text("Reproduce selected simulation")),
                    )
                    .clicked()
                {
                    self.comparison = None;
                    request = self
                        .loaded_folder
                        .clone()
                        .map(|dir| (dir, self.selected.clone()));
                }
                if let Some(r) = &self.comparison {
                    ui.colored_label(
                        if r.passed() {
                            egui::Color32::LIGHT_GREEN
                        } else {
                            egui::Color32::LIGHT_RED
                        },
                        lang.text(if r.passed() {
                            "Reproduction PASS"
                        } else {
                            "Reproduction FAIL"
                        }),
                    );
                    ui.label(lang.text(&format!(
                        "Rays: {} / {}\nray_id mismatches: {}\nstatus mismatches: {}",
                        r.rays,
                        r.expected_rays,
                        r.id_mismatches + r.stored_id_mismatches,
                        r.status_mismatches
                    )));
                    ui.small(lang.text(&format!(
                        "Stored IDs checked: {} / {}",
                        r.stored_ids_checked, r.expected_rays
                    )));
                    if r.initial_conditions_hash_verified {
                        ui.label(lang.text(
                            "Initial conditions SHA-256: PASS; seed regenerated; no stored ray IDs",
                        ));
                    } else if r.stored_ids_checked != r.expected_rays {
                        ui.label(lang.text("Stored IDs NOT VERIFIED (legacy file)"));
                    }
                    for (i, name) in ["u_hit", "v_hit", "t_hit"].iter().enumerate() {
                        ui.label(lang.text(&format!("{name} max error: {:.3e}", r.hit_max_abs[i])));
                    }
                    ui.label(lang.text("Stored / reproduced"));
                    for (i, name) in [
                        "Active",
                        "Detected",
                        "Captured",
                        "Escaped",
                        "NumericalFailure",
                    ]
                    .iter()
                    .enumerate()
                    {
                        ui.label(format!(
                            "{}: {} / {}",
                            lang.text(name),
                            r.expected_counts[i],
                            r.actual_counts[i]
                        ));
                    }
                    collapsing(ui, lang, "Full comparison", |ui| {
                        ui.monospace(lang.text(&r.summary()));
                    });
                }
            });
        request
    }
}
