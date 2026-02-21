use crate::app::SublimeRustApp;
use crate::syntax::{SYNTAX_SET, THEME_SET};
use eframe::egui;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::util::LinesWithEndings;

pub fn render_editor_pane(app: &mut SublimeRustApp, ui: &mut egui::Ui) {
    // ── Tab Bar ──────────────────────────────────────────
    egui::ScrollArea::horizontal()
        .id_source("tab_scroll")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut tab_to_close = None;
                let mut tab_to_activate = None;

                for (idx, path) in app.open_tabs.iter().enumerate() {
                    let is_active = Some(idx) == app.active_tab_index;
                    let is_dirty = app.dirty_files.contains(path);
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string();
                    let display_name = if is_dirty {
                        format!("*{}", file_name)
                    } else {
                        file_name
                    };

                    let bg_color = if is_active {
                        egui::Color32::from_rgb(0x23, 0x23, 0x23)
                    } else {
                        egui::Color32::from_rgb(0x18, 0x18, 0x18)
                    };
                    let text_color = if is_active {
                        egui::Color32::from_rgb(0xcc, 0xcc, 0xcc)
                    } else {
                        egui::Color32::from_rgb(0x88, 0x88, 0x88)
                    };

                    let response = ui.add(
                        egui::Button::new(egui::RichText::new(&display_name).color(text_color))
                            .fill(bg_color)
                            .stroke(egui::Stroke::NONE),
                    );

                    if response.clicked() {
                        tab_to_activate = Some(idx);
                    }

                    // Close button (x)
                    if ui
                        .add(egui::Button::new("x").fill(bg_color).small())
                        .clicked()
                    {
                        tab_to_close = Some(idx);
                    }

                    ui.separator();
                }

                if let Some(idx) = tab_to_activate {
                    app.active_tab_index = Some(idx);
                }

                if let Some(idx) = tab_to_close {
                    let is_dirty = app.dirty_files.contains(&app.open_tabs[idx]);
                    if is_dirty {
                        app.closing_file_index = Some(idx);
                    } else {
                        app.close_tab(idx);
                    }
                }
            });
        });

    ui.separator();

    // ── Editor Pane ──────────────────────────────────────
    if let Some(idx) = app.active_tab_index {
        if let Some(path) = app.open_tabs.get(idx).cloned() {
            if let Some(content) = app.tab_contents.get_mut(&path) {
                egui::ScrollArea::vertical()
                    .id_source("editor_scroll")
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            let line_count = content.chars().filter(|&c| c == '\n').count() + 1;
                            let mut line_numbers = String::new();
                            for i in 1..=line_count {
                                line_numbers.push_str(&format!("{:>3}\n", i));
                            }

                            ui.add(egui::Label::new(
                                egui::RichText::new(line_numbers)
                                    .font(egui::TextStyle::Monospace.resolve(ui.style()))
                                    .color(egui::Color32::from_rgb(0x55, 0x55, 0x55)),
                            ));

                            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            let syntax = SYNTAX_SET
                                .find_syntax_by_extension(extension)
                                .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
                            let theme = &THEME_SET.themes["base16-ocean.dark"];

                            let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                                let mut job = egui::text::LayoutJob::default();
                                let mut highlighter = HighlightLines::new(syntax, theme);
                                for line in LinesWithEndings::from(string) {
                                    let ranges: Vec<(Style, &str)> =
                                        highlighter.highlight_line(line, &SYNTAX_SET).unwrap();
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
                                                font_id: egui::TextStyle::Monospace
                                                    .resolve(ui.style()),
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
                                app.dirty_files.insert(path.clone());
                            }

                            if app.find_scroll_requested {
                                if let Some(range) = output.cursor_range {
                                    let rect = output.galley.pos_from_cursor(&range.primary);
                                    ui.scroll_to_rect(
                                        rect.translate(output.galley_pos.to_vec2()),
                                        Some(egui::Align::Center),
                                    );
                                    app.find_scroll_requested = false;
                                }
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
                                app.cursor_pos = (line, col);
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
}
