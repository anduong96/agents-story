use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::Alignment,
    style::Color,
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{self};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(&mut terminal);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title(Span::styled(
                    " Agents Story ",
                    ratatui::style::Style::default(),
                ))
                .borders(Borders::ALL)
                .border_style(ratatui::style::Style::default().fg(Color::Cyan));

            let paragraph = Paragraph::new("Press 'q' to quit")
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, size);
        })?;

        // Handle events
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}
