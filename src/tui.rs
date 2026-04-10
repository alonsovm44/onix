use crate::models::OnixManifest;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;

/// Displays a TUI confirmation dialog for the installation.
pub fn confirm_install(manifest: &OnixManifest) -> Result<bool> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, manifest);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, manifest: &OnixManifest) -> Result<bool> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(3),    // Permissions
                    Constraint::Length(4), // Destination
                    Constraint::Length(1), // Footer
                ])
                .split(f.size());

            let title = Paragraph::new(format!("Onix Installer: {} v{}", manifest.app, manifest.version))
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Information"));
            f.render_widget(title, chunks[0]);

            let perms: Vec<ListItem> = manifest.permissions
                .iter()
                .map(|p| ListItem::new(format!(" • {}", p)))
                .collect();
            let perms_list = List::new(perms)
                .block(Block::default().borders(Borders::ALL).title("Requested Permissions"))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(perms_list, chunks[1]);

            let dest_text = format!(
                "Installing binary to: {}/{}\nFile type: {}",
                manifest.installation.target_dir,
                manifest.installation.bin_name,
                manifest.installation.file_type
            );
            let dest = Paragraph::new(dest_text)
                .block(Block::default().borders(Borders::ALL).title("Installation Details"))
                .wrap(Wrap { trim: true });
            f.render_widget(dest, chunks[2]);

            let footer = Paragraph::new("Confirm: [ENTER] | Cancel: [ESC / Q]")
                .alignment(Alignment::Center)
                .style(Style::default().add_modifier(Modifier::DIM));
            f.render_widget(footer, chunks[3]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => return Ok(true),
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}