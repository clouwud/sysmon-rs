use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn usage_bar(usage: f32, width: usize) -> String {
    let filled = (usage / 100.0 * width as f32).round() as usize;
    let mut bar = String::new();
    for i in 0..width {
        if i < filled {
            bar.push('█');
        } else {
            bar.push('░');
        }
    }
    bar
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create system object with CPU + memory refresh enabled
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    let tick_rate = Duration::from_millis(1000);
    let mut last_tick = Instant::now();

    loop {
        sys.refresh_cpu();
        sys.refresh_memory();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length((sys.cpus().len() + 2) as u16), // CPU area height
                    Constraint::Length(3), // Memory area height
                    Constraint::Min(0),
                ])
                .split(f.size());

            // CPU usage (per core with bars)
            let mut cpu_lines = Vec::new();
            for (i, cpu) in sys.cpus().iter().enumerate() {
                let usage = cpu.cpu_usage();
                let bar = usage_bar(usage, 20); // 20-char bar
                cpu_lines.push(Line::from(vec![
                    Span::styled(
                        format!("Core {:02}: {:>5.2}% ", i, usage),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(bar, Style::default().fg(Color::Green)),
                ]));
            }
            let cpu_block = Paragraph::new(cpu_lines)
                .block(Block::default().title("CPU Usage per Core").borders(Borders::ALL));
            f.render_widget(cpu_block, chunks[0]);

            // Memory usage
            let total_mem = sys.total_memory();
            let used_mem = sys.used_memory();
            let mem_usage_percent = (used_mem as f64 / total_mem as f64) * 100.0;
            let mem_bar = usage_bar(mem_usage_percent as f32, 30);
            let mem_text = vec![
                Line::from(Span::styled(
                    format!(
                        "Memory: {:.2} / {:.2} GiB ({:.1}%)",
                        used_mem as f64 / 1024.0 / 1024.0,
                        total_mem as f64 / 1024.0 / 1024.0,
                        mem_usage_percent
                    ),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(mem_bar, Style::default().fg(Color::Blue))),
            ];

            let mem_block = Paragraph::new(mem_text)
                .block(Block::default().title("Memory").borders(Borders::ALL));
            f.render_widget(mem_block, chunks[1]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
