pub mod dialogs;
pub mod editor;
pub mod explorer;
pub mod footer;
pub mod menu;

pub use dialogs::render_close_confirmation;
pub use editor::render_editor_pane;
pub use explorer::render_project_explorer;
pub use footer::render_footer;
pub use menu::render_menu_bar;
