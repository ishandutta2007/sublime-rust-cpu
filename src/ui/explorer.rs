use eframe::egui;
use std::path::PathBuf;
use std::fs;
use crate::app::SublimeRustApp;

pub fn render_project_explorer(app: &mut SublimeRustApp, ui: &mut egui::Ui, path: PathBuf) {
    let dir_name = path
        .file_name()
        .map_or("?", |os_str| os_str.to_str().unwrap_or("?"))
        .to_string();

    let is_expanded = app.expanded_dirs.contains(&path);

    let response = egui::CollapsingHeader::new(&dir_name)
        .id_source(&path)
        .open(Some(is_expanded))
        .show(ui, |ui| {
            if let Ok(entries) = fs::read_dir(&path) {
                let mut sorted_entries: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
                sorted_entries.sort_by(|a, b| {
                    let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.file_name().cmp(&b.file_name()),
                    }
                });

                for entry in sorted_entries {
                    let entry_path = entry.path();
                    let file_name = entry.file_name().to_str().unwrap_or("?").to_string();

                    if entry_path.is_dir() {
                        render_project_explorer(app, ui, entry_path);
                    } else {
                        // File entry
                        let is_dirty = app.dirty_files.contains(&entry_path);
                        let display_name = if is_dirty { format!("*{}", file_name) } else { file_name.clone() };
                        
                        if ui.add(egui::Label::new(format!("  {}", display_name)).sense(egui::Sense::click())).clicked() {
                            if let Some(pos) = app.open_tabs.iter().position(|p| p == &entry_path) {
                                app.active_tab_index = Some(pos);
                            } else {
                                if let Ok(content) = fs::read_to_string(&entry_path) {
                                    app.tab_contents.insert(entry_path.clone(), content);
                                    app.open_tabs.push(entry_path.clone());
                                    app.active_tab_index = Some(app.open_tabs.len() - 1);
                                }
                            }
                        }
                    }
                }
            }
        });

    if response.header_response.clicked() {
        if is_expanded {
            app.expanded_dirs.remove(&path);
        } else {
            app.expanded_dirs.insert(path.clone());
        }
    }
}
