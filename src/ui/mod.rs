pub mod explorer;
pub mod menu;
pub mod footer;
pub mod editor;
pub mod dialogs;

pub use explorer::render_project_explorer;
pub use menu::render_menu_bar;
pub use footer::render_footer;
pub use editor::render_editor_pane;
pub use dialogs::render_close_confirmation;
