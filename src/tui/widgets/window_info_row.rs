use ratatui::prelude::*;

pub fn window_info_row<'p>(key: &str, value: &str) -> Line<'p> {
    let span1 = format!("{key}").bold();
    let span2 = format!("{value}").dim();
    return Line::from(vec![span1, span2]);
}
