use ratatui::{Frame, layout::Rect, widgets::Paragraph};

use crate::{global_state::ApiData};



pub fn render_track_bar(frame: &mut Frame, chunk: Rect, api_data: &ApiData) {
    if let Some(stream_data) = &api_data.track_stream {
        let stream_len_p = Paragraph::new(format!("{}", stream_data.len()));
        frame.render_widget(stream_len_p, chunk);
    }
}