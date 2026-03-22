use ratatui::{Frame, layout::Rect, widgets::{Paragraph, Wrap}};

use crate::global_state::ApiData;



pub fn render_test_page(frame: &mut Frame, chunk: Rect, api_data: &ApiData) {
    if let Some(text) = &api_data.track_metadata {
        let _ = std::fs::write("track_metadata_debug.json", text);
        let text_p = Paragraph::new("Metadata written to track_metadata_debug.json".to_string())
            .wrap(Wrap { trim: true });
        frame.render_widget(text_p, chunk);
    }
}