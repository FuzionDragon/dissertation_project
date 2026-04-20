use std::collections::HashMap;

use anyhow::Result;
use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListState};

use crate::level_checker::Level;
use crate::scoring::Rank;

pub fn level_select_tui(levels: &HashMap<String, Level>) -> Result<usize> {
    let mut list_state = ListState::default().with_selected(Some(0));
    ratatui::run(|terminal| {
        loop {
            terminal
                .draw(|frame| render(frame, &mut list_state, levels))
                .expect("Unable to render UI");
            if let Some(key) = event::read()
                .expect("Unable to read key press")
                .as_key_press_event()
            {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => list_state.select_previous(),
                    KeyCode::Enter => break,
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    });

    Ok(list_state.selected().unwrap())
}

fn render(frame: &mut Frame, list_state: &mut ListState, levels: &HashMap<String, Level>) {
    let outer_layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let [top, bottom] = frame.area().layout(&outer_layout);

    let title = Line::from_iter([
        Span::from("Level Select").bold(),
        Span::from(
            " (Press Enter to select desired level, 'q' to quit, and arrow keys to navigate)",
        ),
    ]);
    frame.render_widget(title.centered(), top);
    let mut items: Vec<String> = Vec::new();

    for (id, level) in levels.iter() {
        items.push(format!("{}: {}", id, level.level_title));
    }
    items.sort();
    let list_items = List::new(items)
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");

    let inner_layout =
        Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)]).spacing(1);
    let [list_area, entry] = bottom.layout(&inner_layout);

    frame.render_stateful_widget(list_items, list_area, list_state);
    render_bottom_list(
        frame,
        entry,
        levels
            .get(&list_state.selected().unwrap().to_string())
            .unwrap(),
    );
}

fn render_bottom_list(frame: &mut Frame, area: Rect, level: &Level) {
    let title = Line::from_iter([
        Span::from("Title").bold(),
        Span::from(": "),
        Span::from(level.level_title.clone()),
    ]);
    let intructions = Line::from_iter([
        Span::from("Instructions").bold(),
        Span::from(": "),
        Span::from(level.level_description.clone()),
    ]);
    let level_type = Line::from_iter([
        Span::from("Level Type").bold(),
        Span::from(": "),
        Span::from(level.level_type.as_str()),
    ]);

    let highscore: String;
    if let Some(score) = level.highest_score {
        highscore = score.to_string();
    } else {
        highscore = "N/A".to_string();
    }
    let highscore_line = Line::from_iter([
        Span::from("Highscore").bold(),
        Span::from(": "),
        Span::from(highscore),
    ]);

    let shortest_time: String;
    if let Some(time) = level.highest_score {
        shortest_time = time.to_string();
    } else {
        shortest_time = "N/A".to_string();
    }
    let shortest_time_line = Line::from_iter([
        Span::from("Shortest Time").bold(),
        Span::from(": "),
        Span::from(shortest_time),
    ]);

    let number_of_commands: String;
    if let Some(n_commands) = level.highest_score {
        number_of_commands = n_commands.to_string();
    } else {
        number_of_commands = "N/A".to_string();
    }
    let number_of_commands_line = Line::from_iter([
        Span::from("Number of Commands Used").bold(),
        Span::from(": "),
        Span::from(number_of_commands),
    ]);

    let rank_span: Span;
    if let Some(rank) = level.rank.clone() {
        rank_span = match rank {
            Rank::Gold => Span::from(rank.as_str()).yellow(),
            Rank::Silver => Span::from(rank.as_str()).white(),
            Rank::Bronze => Span::from(rank.as_str()).red(),
        }
    } else {
        rank_span = Span::from("N/A");
    }
    let rank_line = Line::from_iter([Span::from("Rank").bold(), Span::from(": "), rank_span]);

    let text = Text::from(vec![
        title,
        intructions,
        level_type,
        highscore_line,
        shortest_time_line,
        number_of_commands_line,
        rank_line,
    ]);

    frame.render_widget(text, area);
}
