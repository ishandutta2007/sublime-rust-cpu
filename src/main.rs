use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use once_cell::sync::Lazy;

// ── Globals for Syntax Highlighting ─────────────────────────────────────────

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

// ── App State ─────────────────────────────────────────────────────────────────

struct SublimeRustApp {
    // open_menu is handled by egui's immediate mode menu logic mostly, 
    // but we keep the structure if we need custom logic.
    current_dir: PathBuf,
    expanded_dirs: HashSet<PathBuf>,
    open_tabs: Vec<PathBuf>,
    active_tab_index: Option<usize>,
    tab_contents: HashMap<PathBuf, String>,
    dirty_files: HashSet<PathBuf>,
    cursor_pos: (usize, usize),
    closing_file_index: Option<usize>,
    sidebar_visible: bool,
    find_query: String,
    find_matches: Vec<usize>,
    current_match_index: Option<usize>,
    find_active: bool,
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
            dirty_files: HashSet::new(),
            cursor_pos: (1, 1),
            closing_file_index: None,
            sidebar_visible: true,
            find_query: String::new(),
            find_matches: Vec::new(),
            current_match_index: None,
            find_active: false,
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
    fn perform_find(&mut self) {
        self.find_matches.clear();
        if self.find_query.is_empty() {
            self.current_match_index = None;
            return;
        }

        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx) {
                if let Some(content) = self.tab_contents.get(path) {
                    self.find_matches = content.match_indices(&self.find_query).map(|(i, _)| i).collect();
                    if !self.find_matches.is_empty() {
                        if self.current_match_index.is_none() || self.current_match_index.unwrap() >= self.find_matches.len() {
                            self.current_match_index = Some(0);
                        }
                    } else {
                        self.current_match_index = None;
                    }
                }
            }
        }
    }

    fn move_to_match(&mut self, ctx: &egui::Context) {
        if let Some(match_idx) = self.current_match_index {
            if let Some(byte_offset) = self.find_matches.get(match_idx) {
                let editor_id = egui::Id::new("main_editor");
                if let Some(mut state) = egui::text_edit::TextEditState::load(ctx, editor_id) {
                    let ccursor = egui::text::CCursor::new(*byte_offset);
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                    state.store(ctx, editor_id);
                }
            }
        }
    }

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
                            let is_dirty = self.dirty_files.contains(&entry_path);
                            let display_name = if is_dirty { format!("*{}", file_name) } else { file_name.clone() };
                            
                            if ui.add(egui::Label::new(format!("  {}", display_name)).sense(egui::Sense::click())).clicked() {
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

    fn save_file(&mut self, path: PathBuf) {
        if let Some(content) = self.tab_contents.get(&path) {
            if let Ok(_) = fs::write(&path, content) {
                self.dirty_files.remove(&path);
            }
        }
    }

    fn save_active_file(&mut self) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                self.save_file(path);
            }
        }
    }

    fn save_as_active_file(&mut self) {
        if let Some(idx) = self.active_tab_index {
            if let Some(path) = self.open_tabs.get(idx).cloned() {
                if let Some(new_path) = rfd::FileDialog::new()
                    .set_file_name(path.file_name().unwrap().to_str().unwrap())
                    .save_file() 
                {
                    if let Some(content) = self.tab_contents.get(&path).cloned() {
                        if let Ok(_) = fs::write(&new_path, &content) {
                            // Update the tab to the new path
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

    fn save_all_files(&mut self) {
        let dirty: Vec<_> = self.dirty_files.iter().cloned().collect();
        for path in dirty {
            self.save_file(path);
        }
    }

    fn close_tab(&mut self, idx: usize) {
        let path = self.open_tabs.remove(idx);
        // We don't necessarily remove from tab_contents to keep cache, 
        // but we definitely remove from dirty_files as it's no longer "open" in a dirty state we care about for this session
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

    fn render_close_confirmation(&mut self, ctx: &egui::Context) {
        if let Some(idx) = self.closing_file_index {
            let mut open = true;
            let file_name = self.open_tabs[idx].file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            
            egui::Window::new("Unsaved Changes")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!("Do you want to save the changes you made to {}?", file_name));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let path_to_save = self.open_tabs[idx].clone();
                            self.save_file(path_to_save);
                            self.close_tab(idx);
                            self.closing_file_index = None;
                        }
                        if ui.button("Don't Save").clicked() {
                            self.close_tab(idx);
                            self.closing_file_index = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.closing_file_index = None;
                        }
                    });
                });
            
            if !open {
                self.closing_file_index = None;
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
                    if ui.button("Save (Ctrl+S)").clicked() {
                        self.save_active_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As... (Ctrl+Shift+S)").clicked() {
                        self.save_as_active_file();
                        ui.close_menu();
                    }
                    if ui.button("Save All").clicked() {
                        self.save_all_files();
                        ui.close_menu();
                    }
                    ui.separator();
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
                    if ui.button("Find... (Ctrl+F)").clicked() {
                        self.find_active = true;
                        ui.close_menu();
                    }
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

    fn render_footer(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let icon = if self.sidebar_visible { "[<]" } else { "[>]" };
                    if ui.button(icon).on_hover_text("Toggle Side Bar").clicked() {
                        self.sidebar_visible = !self.sidebar_visible;
                    }
                    ui.separator();

                    if self.find_active {
                        ui.label("Find:");
                        let response = ui.add(egui::TextEdit::singleline(&mut self.find_query).desired_width(150.0));
                        if response.changed() {
                            self.perform_find();
                        }
                        if response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            // Focus back to editor or find next? Usually find next.
                        }
                        
                        if !self.find_matches.is_empty() {
                            let curr = self.current_match_index.unwrap_or(0) + 1;
                            ui.label(format!("{} of {} matches", curr, self.find_matches.len()));
                        } else if !self.find_query.is_empty() {
                            ui.label("No matches");
                        }
                    } else {
                        if self.active_tab_index.is_some() {
                            ui.label(format!("Line {}, Col {}", self.cursor_pos.0, self.cursor_pos.1));
                        } else {
                            ui.label("Ready");
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(idx) = self.active_tab_index {
                            if let Some(path) = self.open_tabs.get(idx) {
                                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                let syntax = SYNTAX_SET
                                    .find_syntax_by_extension(extension)
                                    .or_else(|| SYNTAX_SET.find_syntax_by_first_line(extension)) // fallback
                                    .map(|s| s.name.as_str())
                                    .unwrap_or("Plain Text");
                                ui.label(format!("Language: {}", syntax));
                            }
                        } else {
                            ui.label("No file");
                        }

                        if self.find_active {
                            if ui.button("Find Next").clicked() {
                                if !self.find_matches.is_empty() {
                                    let next_idx = (self.current_match_index.unwrap_or(0) + 1) % self.find_matches.len();
                                    self.current_match_index = Some(next_idx);
                                    self.move_to_match(ctx);
                                }
                            }
                            if ui.button("Find Prev").clicked() {
                                if !self.find_matches.is_empty() {
                                    let prev_idx = if self.current_match_index.unwrap_or(0) == 0 {
                                        self.find_matches.len() - 1
                                    } else {
                                        self.current_match_index.unwrap() - 1
                                    };
                                    self.current_match_index = Some(prev_idx);
                                    self.move_to_match(ctx);
                                }
                            }
                            ui.separator();
                        }
                    });
                });
            });
            ui.add_space(3.0);
        });
    }
}

impl eframe::App for SublimeRustApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle shortcuts
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S))) {
            self.save_active_file();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::S))) {
            self.save_as_active_file();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::F))) {
            self.find_active = true;
        }

        self.render_menu_bar(ctx);
        self.render_footer(ctx);
        self.render_close_confirmation(ctx);

        if self.sidebar_visible {
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
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Tab Bar ──────────────────────────────────────────
            egui::ScrollArea::horizontal().id_source("tab_scroll").show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut tab_to_close = None;
                    let mut tab_to_activate = None;

                    for (idx, path) in self.open_tabs.iter().enumerate() {
                        let is_active = Some(idx) == self.active_tab_index;
                        let is_dirty = self.dirty_files.contains(path);
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
                        let display_name = if is_dirty { format!("*{}", file_name) } else { file_name };
                        
                        let bg_color = if is_active { egui::Color32::from_rgb(0x23, 0x23, 0x23) } else { egui::Color32::from_rgb(0x18, 0x18, 0x18) };
                        let text_color = if is_active { egui::Color32::from_rgb(0xcc, 0xcc, 0xcc) } else { egui::Color32::from_rgb(0x88, 0x88, 0x88) };

                        let response = ui.add(
                            egui::Button::new(egui::RichText::new(&display_name).color(text_color))
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
                        let is_dirty = self.dirty_files.contains(&self.open_tabs[idx]);
                        if is_dirty {
                            self.closing_file_index = Some(idx);
                        } else {
                            self.close_tab(idx);
                        }
                    }
                });
            });

            ui.separator();

            // ── Editor Pane ──────────────────────────────────────
            if let Some(idx) = self.active_tab_index {
                if let Some(path) = self.open_tabs.get(idx).cloned() {
                    if let Some(content) = self.tab_contents.get_mut(&path) {
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

                                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                let syntax = SYNTAX_SET
                                    .find_syntax_by_extension(extension)
                                    .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
                                let theme = &THEME_SET.themes["base16-ocean.dark"];

                                let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                                    let mut job = egui::text::LayoutJob::default();
                                    let mut highlighter = HighlightLines::new(syntax, theme);
                                    for line in LinesWithEndings::from(string) {
                                        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &SYNTAX_SET).unwrap();
                                        for (style, text) in ranges {
                                            let color = egui::Color32::from_rgba_unmultiplied(
                                                style.foreground.r,
                                                style.foreground.g,
                                                style.foreground.b,
                                                style.foreground.a,
                                            );
                                            job.append(
                                                text,
                                                0.0,
                                                egui::TextFormat {
                                                    font_id: egui::TextStyle::Monospace.resolve(ui.style()),
                                                    color,
                                                    ..Default::default()
                                                },
                                            );
                                        }
                                    }
                                    ui.fonts(|f| f.layout_job(job))
                                };

                                let min_height = ui.available_height();
                                let output = egui::TextEdit::multiline(content)
                                    .id(egui::Id::new("main_editor"))
                                    .code_editor()
                                    .font(egui::TextStyle::Monospace) // Use monospace font
                                    .desired_width(f32::INFINITY)
                                    .min_size(egui::vec2(0.0, min_height))
                                    .lock_focus(true)
                                    .layouter(&mut layouter)
                                    .show(ui);

                                if output.response.changed() {
                                    self.dirty_files.insert(path.clone());
                                }

                                if let Some(range) = output.cursor_range {
                                    let char_idx = range.primary.ccursor.index;
                                    let mut line = 1;
                                    let mut col = 1;
                                    for (i, c) in content.chars().enumerate() {
                                        if i >= char_idx {
                                            break;
                                        }
                                        if c == '\n' {
                                            line += 1;
                                            col = 1;
                                        } else {
                                            col += 1;
                                        }
                                    }
                                    self.cursor_pos = (line, col);
                                }
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
            .with_title("SuRuC"),
        ..Default::default()
    };
    eframe::run_native(
        "sublime_rust_cpu",
        native_options,
        Box::new(|cc| Box::new(SublimeRustApp::new(cc))),
    )
}
