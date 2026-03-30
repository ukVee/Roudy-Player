use std::sync::atomic::Ordering;

use ratatui::{Frame, layout::{Constraint, Layout, Rect}, style::{Color, Style}, widgets::{Block, Gauge, Paragraph}};

use crate::global_state::{RoudyData};



pub fn render_track_bar(frame: &mut Frame, chunk: Rect, roudy_data: &RoudyData) {
    let (paused, volume ) = &roudy_data.track_controls;
    let block = Block::bordered();
    let inside_borders = block.inner(chunk);
    frame.render_widget(block, chunk);
    if let Some(current_track) = &roudy_data.current_track {
        let current_paused = paused.load(Ordering::Relaxed);
        let current_volume = f32::from_bits(volume.load(Ordering::Relaxed));

        let paused_p = if current_paused  {
            Paragraph::new(">")
        }else {
            Paragraph::new("||")
        };
        
        let title_p = Paragraph::new(format!("{}", &current_track.title));
        let artist_tag_p = Paragraph::new(format!("{}", &current_track.tag_list));

        let duration_sec = current_track.duration / 1000;
        let minutes = duration_sec / 60;
        let seconds = duration_sec % 60;
        let time_p = Paragraph::new(format!("{}:{:02}",minutes.to_string(), seconds.to_string()));

        let volume_guage = Gauge::default()
            .ratio(current_volume as f64)
            .gauge_style(Style::default().fg(Color::Red));

        //main layout gets split into two sections stacked on eachother
        let main_layout = Layout::vertical([
            Constraint::Percentage(50),//upper
            Constraint::Percentage(50),//lower
        ]).split(inside_borders);
        //top has two col, one big one small
        let top_layout = Layout::horizontal([
            Constraint::Fill(1),//left col upper
            Constraint::Length(4),//time
        ]).split(main_layout[0]);
        //top left has two fiels stacked ontop of eachother
        let top_left_col_layout = Layout::horizontal([
            Constraint::Percentage(50),//title
            Constraint::Percentage(50),//artist_tag
        ]).split(top_layout[0]);
        let bottom_layout = Layout::horizontal([
            Constraint::Length(3),//pause/play
            Constraint::Fill(1),//empty for now, will be time passed bar
            Constraint::Length(10)//volume bar
        ]).split(main_layout[1]);
        
        frame.render_widget(title_p, top_left_col_layout[0]);
        frame.render_widget(artist_tag_p, top_left_col_layout[1]);
        frame.render_widget(time_p, top_layout[1]);
        frame.render_widget(paused_p, bottom_layout[0]);
        frame.render_widget(volume_guage, bottom_layout[2]);
    }
}