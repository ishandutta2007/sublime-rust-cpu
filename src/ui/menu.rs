use crate::app::SublimeRustApp;
use eframe::egui;

pub fn render_menu_bar(app: &mut SublimeRustApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New File (Ctrl+N)").clicked() {
                    app.new_file();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Open File... (Ctrl+O)").clicked() {
                    app.open_file();
                    ui.close_menu();
                }
                if ui.button("Open Folder... (Ctrl+Shift+O)").clicked() {
                    app.open_folder();
                    ui.close_menu();
                }
                ui.menu_button("Open Recent", |_| {});
                ui.separator();
                if ui.button("Save (Ctrl+S)").clicked() {
                    app.save_active_file();
                    ui.close_menu();
                }
                if ui.button("Save As... (Ctrl+Shift+S)").clicked() {
                    app.save_as_active_file();
                    ui.close_menu();
                }
                if ui.button("Save All").clicked() {
                    app.save_all_files();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit (Alt+F4)").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo (Ctrl+Z)").clicked() {
                    ui.close_menu();
                }
                if ui.button("Redo (Ctrl+Y)").clicked() {
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Cut (Ctrl+X)").clicked() {
                    ui.close_menu();
                }
                if ui.button("Copy (Ctrl+C)").clicked() {
                    ui.close_menu();
                }
                if ui.button("Paste (Ctrl+V)").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Selection", |ui| {
                if ui.button("Select All (Ctrl+A)").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Find", |ui| {
                if ui.button("Find... (Ctrl+F)").clicked() {
                    app.find_active = true;
                    app.find_just_activated = true;
                    ui.close_menu();
                }
                if ui.button("Find in Files... (Ctrl+Shift+F)").clicked() {
                    app.find_in_files_active = !app.find_in_files_active;
                    if app.find_in_files_active {
                        if let Some(dir) = &app.current_dir {
                            app.find_in_files_where_query = dir.to_str().unwrap_or("").to_string();
                        }
                    }
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                ui.menu_button("Side Bar", |ui| {
                    let label = if app.sidebar_visible { "Hide" } else { "Show" };
                    if ui.button(label).clicked() {
                        app.sidebar_visible = !app.sidebar_visible;
                        ui.close_menu();
                    }
                });
            });

            ui.menu_button("Goto", |ui| {
                if ui.button("Goto Anything... (Ctrl+P)").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Command Palette... (Ctrl+Shift+P)").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Project", |ui| {
                if ui.button("Open Project...").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Preferences", |ui| {
                if ui.button("Settings").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About Sublime Text").clicked() {
                    ui.close_menu();
                }
            });
        });
    });
}
