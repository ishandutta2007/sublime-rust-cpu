use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

// ── App State ─────────────────────────────────────────────────────────────────

struct SublimeRustApp {
    // open_menu is handled by egui's immediate mode menu logic mostly, 
    // but we keep the structure if we need custom logic.
    current_dir: PathBuf,
    expanded_dirs: HashSet<PathBuf>,
    open_tabs: Vec<PathBuf>,
    active_tab_index: Option<usize>,
    tab_contents: HashMap<PathBuf, String>,
    // Sidebar width is handled by egui::SidePanel state implicitly
}

impl Default for SublimeRustApp {
    fn default() -> Self {
        Self {
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            expanded_dirs: HashSet::new(),
            open_tabs: Vec::new(),
            active_tab_index: None,
            tab_contents: HashMap::new(),
        }
    }
}

impl SublimeRustApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize the look and feel to match Sublime Text dark theme
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::from_rgb(0x23, 0x23, 0x23);
        visuals.panel_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x1e);
        
        // Define style for widgets
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x1e);
        visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x3e, 0x3e, 0x3e);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x2d, 0x2d, 0x2d);
        
        // Text colors
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0xcc, 0xcc, 0xcc));
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0xcc, 0xcc, 0xcc));

        cc.egui_ctx.set_visuals(visuals);

        Self::default()
    }

    /// Recursively renders the project explorer tree.
    fn render_project_explorer(&mut self, ui: &mut egui::Ui, path: PathBuf) {
        let dir_name = path
            .file_name()
            .map_or("?", |os_str| os_str.to_str().unwrap_or("?"))
            .to_string();

        let is_expanded = self.expanded_dirs.contains(&path);

        let response = egui::CollapsingHeader::new(&dir_name)
            .id_source(&path)
            .open(Some(is_expanded))
            .show(ui, |ui| {
                if let Ok(entries) = std::fs::read_dir(&path) {
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
                            self.render_project_explorer(ui, entry_path);
                        } else {
                            // File entry
                            if ui.add(egui::Label::new(format!("  {}", file_name)).sense(egui::Sense::click())).clicked() {
                                if let Some(pos) = self.open_tabs.iter().position(|p| p == &entry_path) {
                                    self.active_tab_index = Some(pos);
                                } else {
                                    if let Ok(content) = fs::read_to_string(&entry_path) {
                                        self.tab_contents.insert(entry_path.clone(), content);
                                        self.open_tabs.push(entry_path.clone());
                                        self.active_tab_index = Some(self.open_tabs.len() - 1);
                                    }
                                }
                            }
                        }
                    }
                }
            });

        if response.header_response.clicked() {
            if is_expanded {
                self.expanded_dirs.remove(&path);
            } else {
                self.expanded_dirs.insert(path.clone());
            }
        }
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New File (Ctrl+N)").clicked() { ui.close_menu(); }
                    ui.separator();
                    if ui.button("Open File... (Ctrl+O)").clicked() { ui.close_menu(); }
                    if ui.button("Open Folder...").clicked() { ui.close_menu(); }
                    ui.menu_button("Open Recent", |_| {});
                    ui.separator();
                    if ui.button("Save (Ctrl+S)").clicked() { ui.close_menu(); }
                    if ui.button("Exit (Alt+F4)").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo (Ctrl+Z)").clicked() { ui.close_menu(); }
                    if ui.button("Redo (Ctrl+Y)").clicked() { ui.close_menu(); }
                    ui.separator();
                    if ui.button("Cut (Ctrl+X)").clicked() { ui.close_menu(); }
                    if ui.button("Copy (Ctrl+C)").clicked() { ui.close_menu(); }
                    if ui.button("Paste (Ctrl+V)").clicked() { ui.close_menu(); }
                });

                ui.menu_button("Selection", |ui| {
                    if ui.button("Select All (Ctrl+A)").clicked() { ui.close_menu(); }
                });

                ui.menu_button("Find", |ui| {
                    if ui.button("Find... (Ctrl+F)").clicked() { ui.close_menu(); }
                });

                ui.menu_button("View", |ui| {
                    ui.menu_button("Side Bar", |_| {});
                });

                ui.menu_button("Goto", |ui| {
                    if ui.button("Goto Anything... (Ctrl+P)").clicked() { ui.close_menu(); }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Command Palette... (Ctrl+Shift+P)").clicked() { ui.close_menu(); }
                });

                ui.menu_button("Project", |ui| {
                    if ui.button("Open Project...").clicked() { ui.close_menu(); }
                });

                ui.menu_button("Preferences", |ui| {
                    if ui.button("Settings").clicked() { ui.close_menu(); }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About Sublime Text").clicked() { ui.close_menu(); }
                });
            });
        });
    }
}

impl eframe::App for SublimeRustApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_menu_bar(ctx);

        egui::SidePanel::left("sidebar_panel")
            .resizable(true)
            .default_width(200.0)
            .width_range(50.0..=600.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let root = self.current_dir.clone();
                    self.render_project_explorer(ui, root);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Tab Bar ──────────────────────────────────────────
            egui::ScrollArea::horizontal().id_source("tab_scroll").show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut tab_to_close = None;
                    let mut tab_to_activate = None;

                    for (idx, path) in self.open_tabs.iter().enumerate() {
                        let is_active = Some(idx) == self.active_tab_index;
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
                        
                        let bg_color = if is_active { egui::Color32::from_rgb(0x23, 0x23, 0x23) } else { egui::Color32::from_rgb(0x18, 0x18, 0x18) };
                        let text_color = if is_active { egui::Color32::from_rgb(0xcc, 0xcc, 0xcc) } else { egui::Color32::from_rgb(0x88, 0x88, 0x88) };

                        let response = ui.add(
                            egui::Button::new(egui::RichText::new(&file_name).color(text_color))
                                .fill(bg_color)
                                .stroke(egui::Stroke::NONE)
                        );

                        if response.clicked() {
                            tab_to_activate = Some(idx);
                        }

                        // Close button (x)
                        if ui.add(egui::Button::new("x").fill(bg_color).small()).clicked() {
                            tab_to_close = Some(idx);
                        }
                        
                        ui.separator();
                    }

                    if let Some(idx) = tab_to_activate {
                        self.active_tab_index = Some(idx);
                    }

                    if let Some(idx) = tab_to_close {
                        self.open_tabs.remove(idx);
                        // Safe removal from map requires checking if other tabs use same file? 
                        // Assuming unique paths in tabs, we can remove content if no other tab has it open.
                        // But for simplicity here, we just remove it from the list.
                        // Ideally we should check if we should remove from `tab_contents` too, 
                        // but keeping it cached is fine for now.
                        
                        if let Some(active_idx) = self.active_tab_index {
                            if idx == active_idx {
                                // We closed the active tab
                                self.active_tab_index = if self.open_tabs.is_empty() {
                                    None
                                } else if idx >= self.open_tabs.len() {
                                    Some(self.open_tabs.len() - 1)
                                } else {
                                    Some(idx)
                                };
                            } else if idx < active_idx {
                                // We closed a tab before the active one, so shift index left
                                self.active_tab_index = Some(active_idx - 1);
                            }
                        }
                    }
                });
            });

            ui.separator();

            // ── Editor Pane ──────────────────────────────────────
            if let Some(idx) = self.active_tab_index {
                if let Some(path) = self.open_tabs.get(idx) {
                    if let Some(content) = self.tab_contents.get_mut(path) {
                        egui::ScrollArea::vertical().id_source("editor_scroll").show(ui, |ui| {
                            ui.horizontal_top(|ui| {
                                let line_count = content.chars().filter(|&c| c == '\n').count() + 1;
                                let mut line_numbers = String::new();
                                for i in 1..=line_count {
                                    line_numbers.push_str(&format!("{:>3}\n", i));
                                }

                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(line_numbers)
                                            .font(egui::TextStyle::Monospace.resolve(ui.style()))
                                            .color(egui::Color32::from_rgb(0x55, 0x55, 0x55))
                                    )
                                );

                                ui.add(
                                    egui::TextEdit::multiline(content)
                                        .code_editor()
                                        .font(egui::TextStyle::Monospace) // Use monospace font
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(40)
                                        .lock_focus(true)
                                );
                            });
                        });
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No file open");
                });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Sublime Rust (EGUI)"),
        ..Default::default()
    };
    eframe::run_native(
        "sublime_rust",
        native_options,
        Box::new(|cc| Box::new(SublimeRustApp::new(cc))),
    )
}
