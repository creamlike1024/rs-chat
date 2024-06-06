use crate::app::{App, ConnectEditing, CurrentScreen, Sender};
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

pub fn ui(f: &mut Frame, app: &mut App) {
    let app_name = "rs-chat";
    let main_layout = Layout::default()
        .direction(Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    match app.current_screen {
        CurrentScreen::Connecting => {
            let title_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let title = Paragraph::new(Text::styled(app_name, Style::default().fg(Color::Yellow)))
                .block(title_block);
            f.render_widget(title, main_layout[0]);

            let area = centered_rect(50, 30, f.size());
            let connecting_block = Block::default()
                .title("Connect to server")
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::DarkGray));

            f.render_widget(connecting_block, area);
            let connecting_layout = Layout::default()
                .direction(Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)])
                .margin(1)
                .split(area);
            let mut address_block = Block::default().title("Address").borders(Borders::ALL);
            let mut name_block = Block::default().title("Name").borders(Borders::ALL);
            let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);
            match app.connect_editing {
                ConnectEditing::Address => {
                    address_block = address_block.style(active_style);
                }
                ConnectEditing::Name => {
                    name_block = name_block.style(active_style);
                }
            }
            let address_text = Paragraph::new(format!("{}", app.address)).block(address_block);
            let name_text = Paragraph::new(format!("{}", app.name)).block(name_block);
            f.render_widget(address_text, connecting_layout[0]);
            f.render_widget(name_text, connecting_layout[1]);
        }
        CurrentScreen::Chat => {
            // chat window
            let title_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let connected_text = format!("Connected to: {}", app.address);

            let title_text = Line::from(vec![
                Span::styled(app_name, Style::default().fg(Color::Yellow)),
                Span::styled(
                    " ".repeat(calculate_spaces(
                        f.size().width as usize,
                        app_name,
                        &*connected_text,
                        true,
                    )),
                    Style::default(),
                ),
                Span::styled(connected_text, Style::default().bold()),
            ]);
            let title = Paragraph::new(title_text)
                .alignment(Alignment::Left)
                .block(title_block);
            f.render_widget(title, main_layout[0]);

            let mut message_list_items = Vec::<ListItem>::new();
            for message in app.messages.lock().unwrap().iter().clone() {
                let time_text = message.time.format("%H:%M").to_string();
                match message.sender {
                    Sender::Local => {
                        let left_text = "You: ".to_owned() + &message.text;
                        message_list_items.push(ListItem::new(Line::from(vec![
                            Span::styled("You: ", Style::default().fg(Color::Gray)),
                            Span::styled(format!("{}", message.text), Style::default()),
                            Span::styled(
                                " ".repeat(calculate_spaces(
                                    f.size().width as usize,
                                    &left_text,
                                    &time_text,
                                    false,
                                )),
                                Style::default(),
                            ),
                            Span::styled(time_text, Style::default().fg(Color::Gray)),
                        ])));
                    }
                    Sender::Remote => {
                        let left_text = format!("{}: ", message.sender_name) + &message.text;
                        let sender_name = match message.sender_name.len() {
                            0 => "NO NAME",
                            _ => &message.sender_name,
                        };
                        message_list_items.push(ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{}: ", sender_name),
                                Style::default().bold().fg(Color::Green),
                            ),
                            Span::styled(format!("{}", message.text), Style::default()),
                            Span::styled(
                                " ".repeat(calculate_spaces(
                                    f.size().width as usize,
                                    &left_text,
                                    &time_text,
                                    false,
                                )),
                                Style::default(),
                            ),
                            Span::styled(time_text, Style::default().fg(Color::Gray)),
                        ])));
                    }
                }
            }
            if !message_list_items.is_empty() {
                app.list_state.select(Some(message_list_items.len() - 1));
            }
            let message_list = List::new(message_list_items);
            f.render_stateful_widget(message_list, main_layout[1], &mut app.list_state);

            // chat input
            let input_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());
            let input = Paragraph::new(Text::from(Span::styled(
                format!("> {}", app.editing_text),
                Style::default(),
            )))
            .block(input_block);
            f.render_widget(input, main_layout[2]);
        }
        CurrentScreen::Quiting => {
            let quit_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());
            let quit_paragraph = Paragraph::new(Text::from(Span::styled(
                "Do you want to quit? (y/n)",
                Style::default().bold(),
            )))
            .block(quit_block);
            let area = centered_rect(80, 40, f.size());
            let quit_layout = Layout::default()
                .direction(Vertical)
                .constraints([Constraint::Length(3)])
                .split(area);
            f.render_widget(quit_paragraph, quit_layout[0]);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let layout_x = Layout::default()
        .direction(Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(layout_x[1])[1]
}

fn calculate_spaces(width: usize, left_text: &str, right_text: &str, has_border: bool) -> usize {
    let left_text_width = UnicodeWidthStr::width(left_text);
    let right_text_width = UnicodeWidthStr::width(right_text);
    let total_text_width = left_text_width + right_text_width;

    if has_border {
        if width > total_text_width + 2 {
            width - total_text_width - 2
        } else {
            0
        }
    } else {
        if width > total_text_width {
            width - total_text_width
        } else {
            0
        }
    }
}
