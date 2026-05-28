use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutAreas {
    pub title_bar: Rect,
    pub org_panel: Rect,
    pub repo_panel: Rect,
    pub status_bar: Rect,
}

pub fn compute_layout(area: Rect) -> LayoutAreas {
    let [title_bar, middle, status_bar] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let [org_panel, repo_panel] =
        Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).areas(middle);

    LayoutAreas {
        title_bar,
        org_panel,
        repo_panel,
        status_bar,
    }
}
