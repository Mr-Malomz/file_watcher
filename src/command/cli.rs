use std::{error::Error, fmt::format, iter};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{self},
};
use ratatui::{
    Frame, Terminal,
    backend::{self, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    prelude::Backend,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};

use super::data::{Commands, Provider};

#[derive(Default)]
struct AppState {
    selected_provider: Option<Provider>,
    input_fields: Vec<(String, String)>,
    selected_input_index: usize,
}

pub fn runCli() -> Result<(), Box<dyn Error>> {
    // Initialize terminal
    let mut stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    terminal.clear()?;

    let mut app = AppState::default();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char('q') => break,
                KeyCode::Down => {
                    if app.selected_input_index + 1 < app.input_fields.len() {
                        app.selected_input_index += 1;
                    }
                }
                KeyCode::Up => {
                    if app.selected_input_index > 0 {
                        app.selected_input_index -= 1;
                    }
                }
                KeyCode::Enter => {
                    if app.selected_provider.is_none() {
                        app.selected_provider = Some(Provider::AWS);
                        set_fields_for_providers(&mut app);
                    } else {
                        println!("\nInput complete. Simulate upload logic here.");
                        break;
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(field) = app.input_fields.get_mut(app.selected_input_index) {
                        field.1.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let Some(field) = app.input_fields.get_mut(app.selected_input_index) {
                        field.1.pop();
                    }
                }

                _ => {}
            }
        }
    }

    // let res = run_app(&mut terminal).await;

    // Cleanup terminal on exit
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &AppState) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            iter::repeat(Constraint::Length(3))
                .take(app.input_fields.len())
                .collect::<Vec<_>>(),
        )
        .split(size);

    let title = match app.selected_provider {
        Some(ref provider) => format!("Selected Provider: {:?}", provider),
        None => "Choose a provider (press Enter to select a provider)".to_string(),
    };

    let title_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);
    f.render_widget(Clear, size);
    f.render_widget(title_block, size);

    for (i, (label, value)) in app.input_fields.iter().enumerate() {
        let input_block = Paragraph::new(value.as_str())
            .style(if i == app.selected_input_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .block(Block::default().title(label.clone()).borders(Borders::ALL));

        f.render_widget(input_block, chunks[i + 1]);
    }

    let hint = Paragraph::new("Press 'q' to quit, 'Enter' to select a provider")
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Hint"));
    f.render_widget(hint, chunks[0]);
}

fn set_fields_for_providers(app: &mut AppState) {
    match app.selected_provider {
        Some(Provider::GoogleDrive) => {
            app.input_fields = vec![
                ("Access Token".to_string(), "".to_string()),
                ("File Path".to_string(), "".to_string()),
            ];
        }
        Some(Provider::Dropbox) => {
            app.input_fields = vec![
                ("Access Token".to_string(), "".to_string()),
                ("File Path".to_string(), "".to_string()),
            ];
        }
        Some(Provider::AWS) => {
            app.input_fields = vec![
                ("Bucket Name".to_string(), "".to_string()),
                ("File Path".to_string(), "".to_string()),
                ("Key".to_string(), "".to_string()),
                ("Region".to_string(), "".to_string()),
            ];
        }
        _ => {}
    }

    app.selected_input_index = 0;
}
