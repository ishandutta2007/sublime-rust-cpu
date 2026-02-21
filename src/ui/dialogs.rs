use crate::app::SublimeRustApp;
use eframe::egui;

pub fn render_close_confirmation(app: &mut SublimeRustApp, ctx: &egui::Context) {
    if let Some(idx) = app.closing_file_index {
        let mut open = true;
        let file_name = app.open_tabs[idx]
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        egui::Window::new("Unsaved Changes")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!(
                    "Do you want to save the changes you made to {}?",
                    file_name
                ));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        let path_to_save = app.open_tabs[idx].clone();
                        app.save_file(path_to_save);
                        app.close_tab(idx);
                        app.closing_file_index = None;
                    }
                    if ui.button("Don't Save").clicked() {
                        app.close_tab(idx);
                        app.closing_file_index = None;
                    }
                    if ui.button("Cancel").clicked() {
                        app.closing_file_index = None;
                    }
                });
            });

        if !open {
            app.closing_file_index = None;
        }
    }
}
