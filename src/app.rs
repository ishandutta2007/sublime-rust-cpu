use crate::ui;
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct SublimeRustApp {
    pub current_dir: Option<PathBuf>, // Changed to Option
    pub expanded_dirs: HashSet<PathBuf>,
    pub open_tabs: Vec<PathBuf>,
    pub active_tab_index: Option<usize>,
    pub tab_contents: HashMap<PathBuf, String>,
    pub dirty_files: HashSet<PathBuf>,
    pub cursor_pos: (usize, usize),
    pub closing_file_index: Option<usize>,
    pub sidebar_visible: bool,
    pub find_query: String,
    pub find_matches: Vec<usize>,
    pub current_match_index: Option<usize>,
    pub find_active: bool,
    pub find_just_activated: bool,
    pub find_scroll_requested: bool,
}

impl Default for SublimeRustApp {
    fn default() -> Self {
        Self {
            current_dir: None, // No default directory(it was env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),)
            expanded_dirs: HashSet::new(),
            open_tabs: Vec::new(),
            active_tab_index: None,
            tab_contents: HashMap::new(),
            dirty_files: HashSet::new(),
            cursor_pos: (1, 1),
            closing_file_index: None,
            sidebar_visible: true,
            find_query: String::new(),
            find_matches: Vec::new(),
            current_match_index: None,
            find_active: false,
            find_just_activated: false,
            find_scroll_requested: false,
        }
    }
}

impl SublimeRustApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::from_rgb(0x23, 0x23, 0x23);
        visuals.panel_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x1e);

        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x1e);
        visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x3e, 0x3e, 0x3e);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x2d, 0x2d, 0x2d);

        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0xcc, 0xcc, 0xcc));
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0xcc, 0xcc, 0xcc));

        cc.egui_ctx.set_visuals(visuals);

        Self::default()
    }

    pub fn open_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.current_dir = Some(path);
        }
    }

    pub fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            if self.current_dir.is_none() {
                // If no folder is open, set the parent of the file as the current directory
                if let Some(parent) = path.parent() {
                    self.current_dir = Some(parent.to_path_buf());
                }
            }
            if let Ok(content) = fs::read_to_string(&path) {
                if !self.open_tabs.contains(&path) {
                    self.tab_contents.insert(path.clone(), content);
                    self.open_tabs.push(path.clone());
                    self.active_tab_index = Some(self.open_tabs.len() - 1);
                } else {
                    // File is already open, just switch to it
                    if let Some(pos) = self.open_tabs.iter().position(|p| p == &path) {
                        self.active_tab_index = Some(pos);
                    }
                }
            }
        }
    }

    pub fn perform_find(&mut self) {
        self.find_matches.clear();
        if self.find_query.is_empty() {
            self.current_match_index = None;
            return;
        }

        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx) {
                if let Some(content) = self.tab_contents.get(path) {
                    self.find_matches = content
                        .match_indices(&self.find_query)
                        .map(|(byte_offset, _)| content[..byte_offset].chars().count())
                        .collect();
                    if !self.find_matches.is_empty() {
                        if self.current_match_index.is_none()
                            || self.current_match_index.unwrap() >= self.find_matches.len()
                        {
                            self.current_match_index = Some(0);
                        }
                    } else {
                        self.current_match_index = None;
                    }
                }
            }
        }
    }

    pub fn move_to_match(&mut self, ctx: &egui::Context) {
        if let Some(match_idx) = self.current_match_index {
            if let Some(char_offset) = self.find_matches.get(match_idx) {
                let editor_id = egui::Id::new("main_editor");
                if let Some(mut state) = egui::text_edit::TextEditState::load(ctx, editor_id) {
                    let start = egui::text::CCursor::new(*char_offset);
                    let end =
                        egui::text::CCursor::new(*char_offset + self.find_query.chars().count());
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(start, end)));
                    state.store(ctx, editor_id);
                    ctx.memory_mut(|mem| mem.request_focus(editor_id));
                    self.find_scroll_requested = true;
                }
            }
        }
    }

    pub fn save_file(&mut self, path: PathBuf) {
        if let Some(content) = self.tab_contents.get(&path) {
            if let Ok(_) = fs::write(&path, content) {
                self.dirty_files.remove(&path);
            }
        }
    }

    pub fn save_active_file(&mut self) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                self.save_file(path);
            }
        }
    }

    pub fn save_as_active_file(&mut self) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                if let Some(new_path) = rfd::FileDialog::new()
                    .set_file_name(path.file_name().unwrap().to_str().unwrap())
                    .save_file()
                {
                    if let Some(content) = self.tab_contents.get(&path).cloned() {
                        if let Ok(_) = fs::write(&new_path, &content) {
                            self.tab_contents.remove(&path);
                            self.dirty_files.remove(&path);
                            self.tab_contents.insert(new_path.clone(), content);
                            self.open_tabs[idx] = new_path;
                        }
                    }
                }
            }
        }
    }

    pub fn save_all_files(&mut self) {
        let dirty: Vec<_> = self.dirty_files.iter().cloned().collect();
        for path in dirty {
            self.save_file(path);
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        let path = self.open_tabs.remove(idx);
        self.dirty_files.remove(&path);

        if let Some(active_idx) = self.active_tab_index {
            if idx == active_idx {
                self.active_tab_index = if self.open_tabs.is_empty() {
                    None
                } else if idx >= self.open_tabs.len() {
                    Some(self.open_tabs.len() - 1)
                } else {
                    Some(idx)
                };
            } else if idx < active_idx {
                self.active_tab_index = Some(active_idx - 1);
            }
        }
    }
}

impl eframe::App for SublimeRustApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle shortcuts
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            ))
        }) {
            self.save_active_file();
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            ))
        }) {
            self.save_as_active_file();
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::F,
            ))
        }) {
            self.find_active = true;
            self.find_just_activated = true;
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::O,
            ))
        }) {
            self.open_file();
        }
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::O,
            ))
        }) {
            self.open_folder();
        }
        if self.find_active && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.find_active = false;
        }

        ui::render_menu_bar(self, ctx);
        ui::render_footer(self, ctx);
        ui::render_close_confirmation(self, ctx);

        if self.sidebar_visible {
            if let Some(root) = self.current_dir.clone() {
                egui::SidePanel::left("sidebar_panel")
                    .resizable(true)
                    .default_width(200.0)
                    .width_range(50.0..=600.0)
                    .show(ctx, |ui| {
                        ui.add_space(5.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui::render_project_explorer(self, ui, root);
                        });
                    });
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.current_dir.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a file or folder to start.");
                });
            } else {
                ui::render_editor_pane(self, ui);
            }
        });
    }
}
