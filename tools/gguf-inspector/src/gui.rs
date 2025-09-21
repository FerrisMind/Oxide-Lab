use eframe::egui;
use rfd::FileDialog;
use opener;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gguf_inspector::format::readable_value;
use std::fs::File;
use std::io::Read;

pub struct GgufApp {
    pub metadata: Vec<(String, String)>,
    pub filter: String,
}

impl Default for GgufApp {
    fn default() -> Self {
        Self { metadata: Vec::new(), filter: String::new() }
    }
}

impl eframe::App for GgufApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GGUF Metadata Viewer");
            ui.horizontal(|ui| {
                if ui.button("Load GGUF File").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        match load_gguf_metadata(&path) {
                            Ok(metadata) => self.metadata = metadata,
                            Err(e) => eprintln!("Failed to load: {}", e),
                        }
                    }
                }
                if ui.button("Clear").clicked() { self.metadata.clear(); }
                if ui.button("Export CSV").clicked() {
                    if let Some(path) = FileDialog::new().save_file() {
                        if let Err(e) = export_csv(&self.metadata, &path) { eprintln!("CSV export failed: {}", e); }
                    }
                }
                if ui.button("Export YAML").clicked() {
                    if let Some(path) = FileDialog::new().save_file() {
                        if let Err(e) = export_yaml(&self.metadata, &path) { eprintln!("YAML export failed: {}", e); }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.filter);
                if ui.button("Apply").clicked() { /* filter applied via iterator below */ }
                if ui.button("Clear filter").clicked() { self.filter.clear(); }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (k, v) in self.metadata.iter().filter(|(k, v)| k.contains(&self.filter) || v.contains(&self.filter)) {
                    ui.label(format!("{}:", k));
                    // If value looks binary (non-utf8 or too long), offer Base64 view
                    if v.len() > 1024 || v.contains("\0") {
                        ui.horizontal(|ui| {
                            ui.label("<binary> (long)");
                            if ui.button("View Base64").clicked() {
                                // show in separate window
                                if let Err(e) = show_base64_dialog(v) { eprintln!("Base64 view failed: {}", e); }
                            }
                        });
                    } else {
                        ui.label(v);
                    }
                    ui.separator();
                }
            });
        });
    }
}

fn show_base64_dialog(data: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Encode string as base64 (assume original bytes are the utf-8 of data)
    let b64 = STANDARD.encode(data.as_bytes());
    // Save to temp file and open with default editor
    let tmp = std::env::temp_dir().join("gguf_metadata_base64.txt");
    std::fs::write(&tmp, b64)?;
    opener::open(&tmp)?;
    Ok(())
}

fn export_csv(metadata: &Vec<(String, String)>, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(&["key", "value"])?;
    for (k, v) in metadata { wtr.write_record(&[k, v])?; }
    wtr.flush()?;
    Ok(())
}

fn export_yaml(metadata: &Vec<(String, String)>, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let map: std::collections::HashMap<_, _> = metadata.iter().cloned().collect();
    let yaml = serde_yaml::to_string(&map)?;
    std::fs::write(path, yaml)?;
    Ok(())
}

fn load_gguf_metadata(path: &std::path::Path) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let mut cursor = std::io::Cursor::new(&buf);
    let content = candle::quantized::gguf_file::Content::read(&mut cursor)?;
    let mut out = Vec::new();
    for (k, v) in content.metadata.iter() {
        let s = readable_value(v);
        out.push((k.clone(), s));
    }
    Ok(out)
}


