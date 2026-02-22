use crate::app::SublimeRustApp;
use crate::syntax::SYNTAX_SET;
use eframe::egui;

pub fn render_footer(app: &mut SublimeRustApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            let icon = if app.sidebar_visible { "[<]" } else { "[>]" };
            let response = ui.button(icon);
            let icon_button_width = response.rect.width();
            if response.on_hover_text("Toggle Side Bar").clicked() {
                app.sidebar_visible = !app.sidebar_visible;
            }
            ui.separator();

            if app.find_in_files_active {
                let remaining_width = ui.available_width() - icon_button_width * 15.0;
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Find  :    ");
                        ui.add(egui::TextEdit::singleline(
                            &mut app.find_in_files_find_query,
                        )
                        .desired_width(remaining_width)
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Where:  ");
                        ui.add(egui::TextEdit::singleline(
                            &mut app.find_in_files_where_query,
                        )
                        .desired_width(remaining_width)
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Replace:");
                        ui.add(egui::TextEdit::singleline(
                            &mut app.find_in_files_replace_query,
                        )
                        .desired_width(remaining_width)
                        );
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.vertical(|ui| {
                        if ui.button(" Find ").clicked() {
                            app.perform_find_in_files();
                        }
                        if ui.button("Replace").clicked() {
                            app.perform_replace_in_files();
                        }
                        ui.checkbox(&mut app.find_in_files_respect_gitignore, "Respect .gitignore");
                    });
                });
            } else if app.find_active {
                ui.label("Find:");
                let find_id = ui.make_persistent_id("find_input");
                let remaining_width = ui.available_width() - icon_button_width * 9.0;
                let response = ui.add(
                    egui::TextEdit::singleline(&mut app.find_query)
                        .id(find_id)
                        .desired_width(remaining_width),
                );

                if app.find_just_activated {
                    ui.ctx().memory_mut(|mem| mem.request_focus(find_id));
                    app.find_just_activated = false;
                }

                if response.changed() {
                    app.perform_find();
                }

                if !app.find_matches.is_empty() {
                    let curr = app.current_match_index.unwrap_or(0) + 1;
                    ui.label(format!("{} of {} matches", curr, app.find_matches.len()));
                } else if !app.find_query.is_empty() {
                    ui.label("No matches");
                }

                if ui.button("x").on_hover_text("Close Find").clicked() {
                    app.find_active = false;
                }
            } else {
                if app.active_tab_index.is_some() {
                    ui.label(format!(
                        "Line {}, Col {}",
                        app.cursor_pos.0, app.cursor_pos.1
                    ));
                } else {
                    ui.label("Ready");
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(idx) = app.active_tab_index {
                    if let Some(path) = app.open_tabs.get(idx) {
                        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if extension != "find-results" {
                            let syntax = SYNTAX_SET
                                .find_syntax_by_extension(extension)
                                .or_else(|| SYNTAX_SET.find_syntax_by_first_line(extension)) // fallback
                                .map(|s| s.name.as_str())
                                .unwrap_or("Plain Text");
                            ui.label(format!("Language: {}", syntax));
                        }
                    }
                } else {
                    ui.label("No file");
                }

                if app.find_active {
                    // let find_id = ui.make_persistent_id("find_input");
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !app.find_matches.is_empty() {
                            let next_idx =
                                (app.current_match_index.unwrap_or(0) + 1) % app.find_matches.len();
                            app.current_match_index = Some(next_idx);
                            app.move_to_match(ctx);
                        }
                    }
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {}
                    if ui.button("Find Next").clicked() {
                        if !app.find_matches.is_empty() {
                            let next_idx =
                                (app.current_match_index.unwrap_or(0) + 1) % app.find_matches.len();
                            app.current_match_index = Some(next_idx);
                            app.move_to_match(ctx);
                            // ctx.memory_mut(|mem| mem.request_focus(find_id));
                        }
                    }
                    if ui.button("Find Prev").clicked() {
                        if !app.find_matches.is_empty() {
                            let prev_idx = if app.current_match_index.unwrap_or(0) == 0 {
                                app.find_matches.len() - 1
                            } else {
                                app.current_match_index.unwrap() - 1
                            };
                            app.current_match_index = Some(prev_idx);
                            app.move_to_match(ctx);
                            // ctx.memory_mut(|mem| mem.request_focus(find_id));
                        }
                    }
                    ui.separator();
                }
            });
        });
        ui.add_space(3.0);
    });
}
