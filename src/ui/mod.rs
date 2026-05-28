pub mod clone_dialog;
pub mod error_popup;
pub mod layout;
pub mod org_panel;
pub mod repo_panel;
pub mod status_bar;

use ratatui::Frame;

use crate::state::AppState;

pub fn draw(frame: &mut Frame, state: &mut AppState) {
    let areas = layout::compute_layout(frame.area());

    status_bar::render_title(frame, areas.title_bar, state);
    org_panel::render(frame, areas.org_panel, state);
    repo_panel::render(frame, areas.repo_panel, state);
    status_bar::render(frame, areas.status_bar, state);

    if let Some(dialog) = &state.clone_dialog.clone() {
        clone_dialog::render(frame, dialog);
    }

    if let Some(msg) = &state.error_popup.clone() {
        error_popup::render(frame, msg);
    }
}
