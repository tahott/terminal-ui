use chrono::{Utc, Timelike};
use chrono_tz::{Asia::{Seoul, Taipei}, America::New_York, Europe::London};
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, time::{Duration, Instant}, ops::Mul};
use tui::{
  backend::{Backend, CrosstermBackend},
  layout::{Alignment, Constraint, Direction, Layout},
  widgets::{Block, Borders, Tabs, Gauge},
  Frame, Terminal, style::{Style, Color, Modifier}, text::{Spans, Span},
};

struct App<'a> {
  pub titles: Vec<&'a str>,
  pub index: usize,
  pub progress_milis: f64,
  pub progress_sec: f64,
  pub progress_min: f64
}

impl<'a> App<'a> {
  fn new() -> App<'a> {
    let now = Utc::now();
    let milis = now.timestamp_subsec_millis() as f64;
    let sec = now.second() as f64;
    let min = now.minute() as f64;
    App {
      titles: vec!["Seoul", "New York", "Taipei", "London"],
      index: 0,
      progress_milis: milis,
      progress_sec: sec,
      progress_min: min,
    }
  }

  pub fn next(&mut self) {
    self.index = (self.index + 1) % self.titles.len();
  }

  pub fn previous(&mut self) {
    if self.index > 0 {
      self.index -= 1;
    } else {
      self.index = self.titles.len() - 1;
    }
  }

  pub fn on_tick(&mut self) {
    self.progress_milis += 10.0;
    if self.progress_milis > 999.0 {
      self.progress_milis = 0.0;
    }
    self.progress_sec += 0.01;
    if self.progress_sec > 59.0 {
      self.progress_sec = 0.0;
    }
    self.progress_min += 0.000166667;
    if self.progress_min > 59.0 {
      self.progress_min = 0.0;
    }
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // create app and run it
  let tick_rate = Duration::from_millis(10);
  let app = App::new();
  let res = run_app(&mut terminal, app, tick_rate);

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
  mut app: App,
  tick_rate: Duration,
) -> io::Result<()> {
  let mut last_tick = Instant::now();
  loop {
    terminal.draw(|f| ui(f, &app))?;

    let timeout = tick_rate
      .checked_sub(last_tick.elapsed())
      .unwrap_or_else(|| Duration::from_secs(0));
    if crossterm::event::poll(timeout)? {
      if let Event::Key(key) = event::read()? {
        // if let KeyCode::Char('q') = key.code {
        //   return Ok(());
        // }
        match key.code {
          KeyCode::Char('q') => return Ok(()),
          KeyCode::Right => app.next(),
          KeyCode::Left => app.previous(),
          _ => {}
        }
      }
    }
    if last_tick.elapsed() >= tick_rate {
      app.on_tick();
      last_tick = Instant::now();
    }
}
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
  // Wrapping block for a group
  // Just draw the block and the group on the same area and build the group
  // with at least a margin of 1
  let size = f.size();
  let chunks = Layout::default().direction(Direction::Vertical).margin(4).constraints([Constraint::Length(3), Constraint::Min(0)].as_ref()).split(size);

  let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Black));

  f.render_widget(block, size);

  let titles = app
    .titles
    .iter()
    .map(|t| {
        let (first, rest) = t.split_at(1);
        Spans::from(vec![
            Span::styled(first, Style::default().fg(Color::Yellow)),
            Span::styled(rest, Style::default().fg(Color::Green)),
        ])
    })
    .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);
    
    let kst = Utc::now().with_timezone(&Seoul).format("%Y-%m-%d %H:%M:%S").to_string();
    let edt = Utc::now().with_timezone(&New_York).format("%Y-%m-%d %H:%M:%S").to_string();
    let cst = Utc::now().with_timezone(&Taipei).format("%Y-%m-%d %H:%M:%S").to_string();
    let bst = Utc::now().with_timezone(&London).format("%Y-%m-%d %H:%M:%S").to_string();

    let gauge_chunks = Layout::default().direction(Direction::Vertical).constraints(
      [Constraint::Percentage(25), Constraint::Percentage(25)].as_ref(),
    ).split(chunks[1]);
    let gauge_sec = Gauge::default().block(Block::default()).percent(app.progress_sec as u16);
    f.render_widget(gauge_sec, gauge_chunks[0]);
    
    let inner = match app.index {
        0 => Block::default().title(kst).title_alignment(Alignment::Center),
        1 => Block::default().title(edt).title_alignment(Alignment::Center),
        2 => Block::default().title(cst).title_alignment(Alignment::Center),
        3 => Block::default().title(bst).title_alignment(Alignment::Center),
        _ => unreachable!(),
    };
    f.render_widget(inner, chunks[1]);
}