use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::error::Error;
use std::io;
use std::time::Duration;
mod app;
mod ui;
use crate::app::{App, ConnectEditing, CurrentScreen};
use crate::ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let ret = run_app(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    match ret {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal
            .draw(|f| ui(f, app))
            .expect("failed to draw window");

        // Key events
        if event::poll(Duration::from_millis(16)).expect("failed to poll key event") {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                match app.current_screen {
                    CurrentScreen::Connecting => {
                        match key.code {
                            KeyCode::Esc => {
                                return Ok(());
                            }
                            KeyCode::Enter => match app.connect_editing {
                                ConnectEditing::Address => {
                                    app.connect_editing = ConnectEditing::Name;
                                }
                                ConnectEditing::Name => {
                                    app.connect()?;
                                }
                            },
                            KeyCode::Tab | KeyCode::Up | KeyCode::Down => {
                                // toggle between address and name
                                match app.connect_editing {
                                    ConnectEditing::Address => {
                                        app.connect_editing = ConnectEditing::Name;
                                    }
                                    ConnectEditing::Name => {
                                        app.connect_editing = ConnectEditing::Address;
                                    }
                                }
                            }
                            KeyCode::Backspace => match app.connect_editing {
                                ConnectEditing::Address => {
                                    app.address.pop();
                                }
                                ConnectEditing::Name => {
                                    app.name.pop();
                                }
                            },
                            KeyCode::Char(value) => match app.connect_editing {
                                ConnectEditing::Address => {
                                    app.address.push(value);
                                }
                                ConnectEditing::Name => {
                                    app.name.push(value);
                                }
                            },
                            _ => {}
                        }
                    }
                    CurrentScreen::Chat => match key.code {
                        KeyCode::Esc => {
                            app.quit();
                        }
                        KeyCode::Enter => {
                            app.send_message();
                        }
                        KeyCode::Char(c) => {
                            app.editing_text.push(c);
                        }
                        KeyCode::Backspace => {
                            app.editing_text.pop();
                        }
                        _ => {}
                    },
                    CurrentScreen::Quiting => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(()),
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            app.current_screen = CurrentScreen::Chat
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
