use std::{error::Error, iter};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use super::data::Provider;

#[derive(Debug, Clone)]
enum AppMode {
    SelectingProvider,
    FillingFields,
}

#[derive(Default)]
struct AppState {
    mode: AppMode,
    available_providers: Vec<Provider>,
    selected_provider_index: usize,
    selected_provider: Option<Provider>,
    input_fields: Vec<(String, String)>,
    selected_input_index: usize,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::SelectingProvider
    }
}


pub fn runCli() -> Result<(), Box<dyn Error>> {
    // Initialize terminal
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    terminal.clear()?;

    let mut app = AppState {
        mode: AppMode::SelectingProvider,
        available_providers: vec![Provider::AWS, Provider::GoogleDrive, Provider::Dropbox],
        selected_provider_index: 0,
        ..Default::default()
    };

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                AppMode::SelectingProvider => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if app.selected_provider_index + 1 < app.available_providers.len() {
                            app.selected_provider_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        if app.selected_provider_index > 0 {
                            app.selected_provider_index -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        app.selected_provider =
                            Some(app.available_providers[app.selected_provider_index].clone());
                        set_fields_for_providers(&mut app);
                        app.mode = AppMode::FillingFields;
                    }
                    _ => {}
                },
                AppMode::FillingFields => {
                    match key.code {
                        KeyCode::Esc => {
                            // Go back to provider selection
                            app.mode = AppMode::SelectingProvider;
                            app.selected_provider = None;
                            app.input_fields.clear();
                            app.selected_input_index = 0;
                        }
                        KeyCode::Char('q') => break,
                        KeyCode::Down => {
                            if !app.input_fields.is_empty()
                                && app.selected_input_index + 1 < app.input_fields.len()
                            {
                                app.selected_input_index += 1;
                            }
                        }
                        KeyCode::Up => {
                            if app.selected_input_index > 0 {
                                app.selected_input_index -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            // Validate that all fields are filled
                            let all_filled = app
                                .input_fields
                                .iter()
                                .all(|(_, value)| !value.trim().is_empty());
                            if all_filled {
                                println!("\nInput complete. Processing upload...");
                                print_collected_data(&app);
                                break;
                            } else {
                                // You could add error handling here, for now just continue
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Some(field) = app.input_fields.get_mut(app.selected_input_index)
                            {
                                field.1.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(field) = app.input_fields.get_mut(app.selected_input_index)
                            {
                                field.1.pop();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Cleanup terminal on exit
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &AppState) {
    let size = f.size();

    match app.mode {
        AppMode::SelectingProvider => {
            // Layout for provider selection
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(5),    // Provider list
                    Constraint::Length(3), // Hint
                ])
                .split(size);

            // Clear the screen
            f.render_widget(Clear, size);

            // Title
            let title_block = Block::default()
                .title("Select a Cloud Storage Provider")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain);
            f.render_widget(title_block, chunks[0]);

            // Provider list
            let provider_items: Vec<ListItem> = app
                .available_providers
                .iter()
                .enumerate()
                .map(|(i, provider)| {
                    let content = match provider {
                        Provider::AWS => "AWS S3",
                        Provider::GoogleDrive => "Google Drive",
                        Provider::Dropbox => "Dropbox",
                    };
                    let style = if i == app.selected_provider_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(content).style(style)
                })
                .collect();

            let provider_list = List::new(provider_items)
                .block(
                    Block::default()
                        .title("Available Providers")
                        .borders(Borders::ALL),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("→ ");

            let mut list_state = ListState::default();
            list_state.select(Some(app.selected_provider_index));
            f.render_stateful_widget(provider_list, chunks[1], &mut list_state);

            // Hint
            let hint = Paragraph::new("Use ↑↓ to navigate, Enter to select, 'q' to quit")
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(hint, chunks[2]);
        }
        AppMode::FillingFields => {
            // Layout for input fields
            let mut constraints = vec![Constraint::Length(3)]; // For title
            constraints.extend(iter::repeat(Constraint::Length(3)).take(app.input_fields.len())); // For input fields
            constraints.push(Constraint::Length(3)); // For hint

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(constraints)
                .split(size);

            // Clear the screen
            f.render_widget(Clear, size);

            // Title
            let title = match app.selected_provider {
                Some(Provider::AWS) => "Configure AWS S3 Settings",
                Some(Provider::GoogleDrive) => "Configure Google Drive Settings",
                Some(Provider::Dropbox) => "Configure Dropbox Settings",
                None => "Configure Settings",
            };

            let title_block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Plain);
            f.render_widget(title_block, chunks[0]);

            // Render input fields
            for (i, (label, value)) in app.input_fields.iter().enumerate() {
                let display_value = if value.is_empty() {
                    format!("Enter {}", label.to_lowercase())
                } else {
                    value.clone()
                };

                let input_block = Paragraph::new(display_value.as_str())
                    .style(if i == app.selected_input_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else if value.is_empty() {
                        Style::default().fg(Color::Gray)
                    } else {
                        Style::default().fg(Color::Green)
                    })
                    .block(Block::default().title(label.clone()).borders(Borders::ALL));

                f.render_widget(input_block, chunks[i + 1]);
            }

            // Hint
            let hint = Paragraph::new("Use ↑↓ to navigate fields, type to input, Enter to submit, Esc to go back, 'q' to quit")
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(hint, chunks[chunks.len() - 1]);
        }
    }
}

fn set_fields_for_providers(app: &mut AppState) {
    match app.selected_provider {
        Some(Provider::GoogleDrive) => {
            app.input_fields = vec![
                ("Access Token".to_string(), "".to_string()),
                ("Path to File".to_string(), "".to_string()),
            ];
        }
        Some(Provider::Dropbox) => {
            app.input_fields = vec![
                ("Access Token".to_string(), "".to_string()),
                ("Path to File".to_string(), "".to_string()),
                ("Key".to_string(), "".to_string()),
            ];
        }
        Some(Provider::AWS) => {
            app.input_fields = vec![
                ("Region".to_string(), "".to_string()),
                ("Bucket Name".to_string(), "".to_string()),
                ("Path to File".to_string(), "".to_string()),
                ("Key".to_string(), "".to_string()),
            ];
        }
        _ => {}
    }

    app.selected_input_index = 0;
}

fn print_collected_data(app: &AppState) {
    println!("Selected Provider: {:?}", app.selected_provider);
    println!("Configuration:");
    for (label, value) in &app.input_fields {
        println!("  {}: {}", label, value);
    }
}
