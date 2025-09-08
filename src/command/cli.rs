use std::{error::Error, iter};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::provider::{
    aws_s3::upload_file_to_s3, dropbox::upload_file_to_dropbox,
    google_drive::upload_file_to_google_drive,
};
use crate::watcher::{
    aws_s3::list_s3_objects, dropbox::list_dropbox_files,
    google_drive::list_google_drive_files,
};

use super::data::{Commands, Provider, Operation};

#[derive(Debug, Clone)]
enum AppMode {
    SelectingOperation,
    SelectingProvider,
    FillingFields,
    DisplayingResults,
}

#[derive(Default)]
struct AppState {
    mode: AppMode,
    available_operations: Vec<Operation>,
    selected_operation_index: usize,
    selected_operation: Option<Operation>,
    available_providers: Vec<Provider>,
    selected_provider_index: usize,
    selected_provider: Option<Provider>,
    input_fields: Vec<(String, String)>,
    selected_input_index: usize,
    list_results: Option<String>, // Store formatted list results for display
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::SelectingOperation
    }
}

pub fn run_cli() -> Result<(), Box<dyn Error>> {
    // initialize terminal
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    terminal.clear()?;

    let mut app = AppState {
        mode: AppMode::SelectingOperation,
        available_operations: vec![Operation::Upload, Operation::ListFiles],
        selected_operation_index: 0,
        available_providers: vec![Provider::AWS, Provider::GoogleDrive, Provider::Dropbox],
        selected_provider_index: 0,
        ..Default::default()
    };

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                AppMode::SelectingOperation => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if app.selected_operation_index + 1 < app.available_operations.len() {
                            app.selected_operation_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        if app.selected_operation_index > 0 {
                            app.selected_operation_index -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        app.selected_operation =
                            Some(app.available_operations[app.selected_operation_index].clone());
                        app.mode = AppMode::SelectingProvider;
                        app.selected_provider_index = 0; // Reset provider selection
                    }
                    _ => {}
                },
                AppMode::SelectingProvider => match key.code {
                    KeyCode::Esc => {
                        // Go back to operation selection
                        app.mode = AppMode::SelectingOperation;
                        app.selected_operation = None;
                    }
                    KeyCode::Char('q') => break,
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
                        
                        match app.selected_operation {
                            Some(Operation::Upload) => {
                                set_fields_for_upload(&mut app);
                                app.mode = AppMode::FillingFields;
                            },
                            Some(Operation::ListFiles) => {
                                set_fields_for_list(&mut app);
                                app.mode = AppMode::FillingFields;
                            },
                            None => {}
                        }
                    }
                    _ => {}
                },
                AppMode::FillingFields => {
                    match key.code {
                        KeyCode::Esc => {
                            // go back to provider selection
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
                            // validate that all fields are filled
                            let all_filled = app
                                .input_fields
                                .iter()
                                .all(|(_, value)| !value.trim().is_empty());

                            if all_filled {
                                let rt = tokio::runtime::Runtime::new()?;
                                match app.selected_operation {
                                    Some(Operation::Upload) => {
                                        rt.block_on(handle_upload_operation(&app));
                                        crossterm::terminal::disable_raw_mode()?;
                                        break;
                                    },
                                    Some(Operation::ListFiles) => {
                                        let list_result = rt.block_on(handle_list_operation(&app));
                                        match list_result {
                                            Ok(formatted_results) => {
                                                app.list_results = Some(formatted_results);
                                                app.mode = AppMode::DisplayingResults;
                                            },
                                            Err(e) => {
                                                eprintln!("❌ Failed to list files: {}", e);
                                                crossterm::terminal::disable_raw_mode()?;
                                                break;
                                            }
                                        }
                                    },
                                    None => continue,
                                }
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
                },
                AppMode::DisplayingResults => {
                    match key.code {
                        KeyCode::Esc => {
                            // Go back to provider selection
                            app.mode = AppMode::SelectingProvider;
                            app.list_results = None;
                            app.input_fields.clear();
                        }
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }
    }

    // cleanup terminal on exit
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &AppState) {
    let size = f.area();

    match app.mode {
        AppMode::SelectingOperation => {
            // Layout for operation selection
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(5),    // Operation list
                    Constraint::Length(3), // Hint
                ])
                .split(size);

            // Clear the screen
            f.render_widget(Clear, size);

            // Title
            let title_block = Block::default()
                .title("📜 File Manager - Select Operation")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain);
            f.render_widget(title_block, chunks[0]);

            // Operation list
            let operation_items: Vec<ListItem> = app
                .available_operations
                .iter()
                .enumerate()
                .map(|(i, operation)| {
                    let content = match operation {
                        Operation::Upload => "📎 Upload Files to Cloud",
                        Operation::ListFiles => "📁 List Files from Cloud",
                    };
                    let style = if i == app.selected_operation_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(content).style(style)
                })
                .collect();

            let operation_list = List::new(operation_items)
                .block(
                    Block::default()
                        .title("Available Operations")
                        .borders(Borders::ALL),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("→ ");

            let mut list_state = ListState::default();
            list_state.select(Some(app.selected_operation_index));
            f.render_stateful_widget(operation_list, chunks[1], &mut list_state);

            // Hint
            let hint = Paragraph::new("Use ↑↓ to navigate, Enter to select, 'q' to quit")
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(hint, chunks[2]);
        },
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

            // Title with selected operation
            let operation_name = match app.selected_operation {
                Some(Operation::Upload) => "Upload",
                Some(Operation::ListFiles) => "List Files",
                None => "Unknown",
            };
            let title = format!("🎯 {} - Select Provider", operation_name);
            let title_block = Block::default()
                .title(title)
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
            let hint = Paragraph::new("Use ↑↓ to navigate, Enter to select, Esc to go back, 'q' to quit")
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
            let operation_name = match app.selected_operation {
                Some(Operation::Upload) => "Upload",
                Some(Operation::ListFiles) => "List",
                None => "Configure",
            };
            let provider_name = match app.selected_provider {
                Some(Provider::AWS) => "AWS S3",
                Some(Provider::GoogleDrive) => "Google Drive",
                Some(Provider::Dropbox) => "Dropbox",
                None => "Provider",
            };
            let title = format!("⚙️ {} {} - Configuration", operation_name, provider_name);

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
        },
        AppMode::DisplayingResults => {
            // Layout for displaying results
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(10),   // Results
                    Constraint::Length(3), // Hint
                ])
                .split(size);

            // Clear the screen
            f.render_widget(Clear, size);

            // Title
            let provider_name = match app.selected_provider {
                Some(Provider::AWS) => "AWS S3",
                Some(Provider::GoogleDrive) => "Google Drive",
                Some(Provider::Dropbox) => "Dropbox",
                None => "Provider",
            };
            let title = format!("📄 {} - File List Results", provider_name);
            let title_block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Plain);
            f.render_widget(title_block, chunks[0]);

            // Results
            let default_text = "No results available.".to_string();
            let results_text = app.list_results.as_ref().unwrap_or(&default_text);
            let results_paragraph = Paragraph::new(results_text.as_str())
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Files"))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(results_paragraph, chunks[1]);

            // Hint
            let hint = Paragraph::new("Press Esc to go back to provider selection, 'q' to quit")
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(hint, chunks[2]);
        }
    }
}

fn set_fields_for_upload(app: &mut AppState) {
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

fn set_fields_for_list(app: &mut AppState) {
    match app.selected_provider {
        Some(Provider::GoogleDrive) => {
            app.input_fields = vec![
                ("Access Token".to_string(), "".to_string()),
                ("Max Results (default: 100)".to_string(), "100".to_string()),
                ("Search Query (optional)".to_string(), "".to_string()),
            ];
        }
        Some(Provider::Dropbox) => {
            app.input_fields = vec![
                ("Access Token".to_string(), "".to_string()),
                ("Folder Path (default: root)".to_string(), "".to_string()),
                ("Max Results (default: 100)".to_string(), "100".to_string()),
            ];
        }
        Some(Provider::AWS) => {
            app.input_fields = vec![
                ("Region".to_string(), "".to_string()),
                ("Bucket Name".to_string(), "".to_string()),
                ("Prefix (optional)".to_string(), "".to_string()),
                ("Max Keys (default: 100)".to_string(), "100".to_string()),
            ];
        }
        _ => {}
    }

    app.selected_input_index = 0;
}

async fn handle_upload_operation(app: &AppState) {
    match app.selected_provider {
        Some(Provider::AWS) => {
            match upload_file_to_s3(
                &app.input_fields[1].1,  // bucket_name
                &app.input_fields[2].1,  // path_to_file
                &app.input_fields[3].1,  // key
                &app.input_fields[0].1,  // region
            ).await {
                Ok(response) => {
                    println!("✅ AWS S3 upload successful!");
                    println!("   Message: {}", response.message);
                },
                Err(e) => eprintln!("❌ Failed to upload file to AWS S3: {}", e),
            }
        }
        Some(Provider::GoogleDrive) => {
            match upload_file_to_google_drive(
                &app.input_fields[0].1,  // access_token
                &app.input_fields[1].1,  // path_to_file
            ).await {
                Ok(response) => {
                    println!("✅ Google Drive upload successful!");
                    println!("   File ID: {}", response.id);
                    println!("   File Name: {}", response.name);
                    if let Some(link) = response.web_view_link {
                        println!("   View Link: {}", link);
                    }
                },
                Err(e) => eprintln!("❌ Failed to upload file to Google Drive: {}", e),
            }
        }
        Some(Provider::Dropbox) => {
            match upload_file_to_dropbox(
                &app.input_fields[0].1,  // access_token
                &app.input_fields[1].1,  // path_to_file
            ).await {
                Ok(response) => {
                    println!("✅ Dropbox upload successful!");
                    println!("   File Name: {}", response.name);
                    println!("   Dropbox Path: {}", response.path_display);
                    println!("   File ID: {}", response.id);
                    println!("   Size: {} bytes", response.size);
                },
                Err(e) => eprintln!("❌ Failed to upload file to Dropbox: {}", e),
            }
        }
        _ => {}
    }
}

async fn handle_list_operation(app: &AppState) -> Result<String, Box<dyn std::error::Error>> {
    match app.selected_provider {
        Some(Provider::AWS) => {
            let max_keys: i32 = app.input_fields[3].1.parse().unwrap_or(100);
            let response = list_s3_objects(
                &app.input_fields[1].1,  // bucket_name
                &app.input_fields[0].1,  // region
                &app.input_fields[2].1,  // prefix
                max_keys,
            ).await?;
            
            let mut result = format!("Found {} objects:\n\n", response.total_count);
            for obj in response.objects {
                result.push_str(&format!(
                    "📄 {}\n   Size: {} bytes\n   Modified: {}\n   Storage Class: {}\n\n",
                    obj.key,
                    obj.size,
                    obj.last_modified,
                    obj.storage_class.unwrap_or_else(|| "Standard".to_string())
                ));
            }
            Ok(result)
        }
        Some(Provider::GoogleDrive) => {
            let max_results: u32 = app.input_fields[1].1.parse().unwrap_or(100);
            let response = list_google_drive_files(
                &app.input_fields[0].1,  // access_token
                max_results,
                &app.input_fields[2].1,  // query
            ).await?;
            
            let mut result = format!("Found {} files:\n\n", response.total_count);
            for file in response.files {
                result.push_str(&format!(
                    "📄 {}\n   ID: {}\n   Size: {}\n   Type: {}\n   Modified: {}\n\n",
                    file.name,
                    file.id,
                    file.size.unwrap_or_else(|| "Unknown".to_string()),
                    file.mime_type.unwrap_or_else(|| "Unknown".to_string()),
                    file.modified_time.unwrap_or_else(|| "Unknown".to_string())
                ));
            }
            Ok(result)
        }
        Some(Provider::Dropbox) => {
            let limit: u32 = app.input_fields[2].1.parse().unwrap_or(100);
            let response = list_dropbox_files(
                &app.input_fields[0].1,  // access_token
                &app.input_fields[1].1,  // path
                limit,
            ).await?;
            
            let mut result = format!("Found {} entries:\n\n", response.total_count);
            for entry in response.entries {
                let icon = if entry.tag == "folder" { "📁" } else { "📄" };
                let size_str = entry.size
                    .map(|s| format!("{} bytes", s))
                    .unwrap_or_else(|| "N/A".to_string());
                result.push_str(&format!(
                    "{} {}\n   Path: {}\n   Size: {}\n   Modified: {}\n\n",
                    icon,
                    entry.name,
                    entry.path_display,
                    size_str,
                    entry.server_modified.unwrap_or_else(|| "Unknown".to_string())
                ));
            }
            Ok(result)
        }
        _ => Err("No provider selected".into())
    }
}

async fn handle_upload(cmd: Commands) {
    match cmd {
        Commands::AWS {
            region,
            bucket_name,
            path_to_file,
            key,
        } => {
            // handle AWS upload logic here
            match upload_file_to_s3(&bucket_name, &path_to_file, &key, &region).await {
                Ok(response) => {
                    println!("✅ AWS S3 upload successful!");
                    println!("   Message: {}", response.message);
                },
                Err(e) => eprintln!("❌ Failed to upload file to AWS S3: {}", e),
            }
        }
        Commands::GoogleDrive {
            access_token,
            path_to_file,
        } => {
            // handle Google Drive upload logic here
            match upload_file_to_google_drive(&access_token, &path_to_file).await {
                Ok(response) => {
                    println!("✅ Google Drive upload successful!");
                    println!("   File ID: {}", response.id);
                    println!("   File Name: {}", response.name);
                    if let Some(link) = response.web_view_link {
                        println!("   View Link: {}", link);
                    }
                },
                Err(e) => eprintln!("❌ Failed to upload file to Google Drive: {}", e),
            }
        }
        Commands::Dropbox {
            access_token,
            path_to_file,
        } => {
            // handle Dropbox upload logic here
            match upload_file_to_dropbox(&access_token, &path_to_file).await {
                Ok(response) => {
                    println!("✅ Dropbox upload successful!");
                    println!("   File Name: {}", response.name);
                    println!("   Dropbox Path: {}", response.path_display);
                    println!("   File ID: {}", response.id);
                    println!("   Size: {} bytes", response.size);
                },
                Err(e) => eprintln!("❌ Failed to upload file to Dropbox: {}", e),
            }
        }
    }
}
