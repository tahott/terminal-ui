use chrono::{Local, Utc};
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, time::{Duration, Instant}};
use tui::{
  backend::{Backend, CrosstermBackend},
  layout::{Alignment, Constraint, Direction, Layout},
  widgets::{Block, BorderType, Borders},
  Frame, Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // create app and run it
  let tick_rate = Duration::from_millis(250);
  let res = run_app(&mut terminal, tick_rate);

  // restore terminal
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    println!("{:?}", err)
  }

  Ok(())
}

fn run_app<B: Backend>(
  terminal: &mut Terminal<B>,
  tick_rate: Duration,
) -> io::Result<()> {
  let mut last_tick = Instant::now();
  loop {
    terminal.draw(|f| ui(f))?;

    let timeout = tick_rate
      .checked_sub(last_tick.elapsed())
      .unwrap_or_else(|| Duration::from_secs(0));
    if crossterm::event::poll(timeout)? {
      if let Event::Key(key) = event::read()? {
        if let KeyCode::Char('q') = key.code {
          return Ok(());
        }
      }
    }
    if last_tick.elapsed() >= tick_rate {
      last_tick = Instant::now();
    }
}
}

fn ui<B: Backend>(f: &mut Frame<B>) {
  // Wrapping block for a group
  // Just draw the block and the group on the same area and build the group
  // with at least a margin of 1
  let size = f.size();

  // Surrounding block
  let block = Block::default()
    .borders(Borders::ALL)
    .title(" Main block with round corners ")
    .title_alignment(Alignment::Center)
    .border_type(BorderType::Rounded);
  f.render_widget(block, size);

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(4)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
    .split(f.size());

  let local_now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
  let block = Block::default()
    .title(format!("Local now is {}", local_now));
  f.render_widget(block, chunks[0]);

  let utc_now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
  let block = Block::default()
    .title(format!("Utc now is {}", utc_now));
  f.render_widget(block, chunks[1]);
}