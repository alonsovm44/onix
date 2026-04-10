use crate::models::OnixManifest;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn display_manifest_tui(manifest: OnixManifest) -> Result<(), Box<dyn std::error::Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, manifest);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    manifest: OnixManifest,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &manifest))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') | KeyCode::Esc = key.code {
                return Ok(());
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, manifest: &OnixManifest) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    // 1. Header
    let title = Paragraph::new(format!("📦 Onix Manifest: {} v{}", manifest.app, manifest.version))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // 2. Main Content (Split horizontally)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left side: Installation & Platforms
    let mut install_text = vec![
        Line::from(vec![Span::styled("Target Directory: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&manifest.installation.target_dir)]),
        Line::from(vec![Span::styled("Binary Name:      ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&manifest.installation.bin_name)]),
        Line::from(""),
        Line::from(Span::styled("Supported Platforms:", Style::default().fg(Color::Yellow))),
    ];

    for source in &manifest.install_on {
        install_text.push(Line::from(format!(" • {} ({})", source.os, source.arch)));
    }

    let install_info = Paragraph::new(install_text)
        .block(Block::default().title(" Details ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(install_info, main_chunks[0]);

    // Right side: Permissions
    let mut perm_text = vec![];
    if manifest.permissions.is_empty() {
        perm_text.push(Line::from("No special permissions requested."));
    } else {
        for perm in &manifest.permissions {
            let detail = match (&perm.path, &perm.variable) {
                (Some(p), _) => format!(" -> {}", p),
                (_, Some(v)) => format!(" -> {}", v),
                _ => "".to_string(),
            };
            perm_text.push(Line::from(vec![
                Span::styled(format!("⚠ {:<11} ", perm.permission_type), Style::default().fg(Color::Red)),
                Span::styled(format!("[{}]", perm.action), Style::default().fg(Color::Gray)),
                Span::raw(detail),
            ]));
        }
    }

    let perms_info = Paragraph::new(perm_text)
        .block(Block::default().title(" Required Permissions ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(perms_info, main_chunks[1]);

    // 3. Footer
    let message_text = manifest.message.clone().unwrap_or_else(|| "Press 'q' to exit".to_string());
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Message: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(message_text),
        Span::raw(" | "),
        Span::styled("Press 'q' or 'Esc' to exit", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
    ]))
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::TOP));
    
    f.render_widget(footer, chunks[2]);
}