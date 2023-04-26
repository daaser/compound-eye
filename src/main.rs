use std::{
  io,
  time::{Duration, Instant},
};

use crossterm::{
  event::{self, Event, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
  backend::{Backend, CrosstermBackend},
  layout::Rect,
  widgets::Paragraph,
  Frame, Terminal,
};

const C: f64 = 1.45;
const Q: f64 = 0.5;
const G: f64 = 0.25;
const P: f64 = 0.25;

fn main() -> anyhow::Result<()> {
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let size = terminal.size()?;
  if size.width < 74 || size.height < 21 {
    quit(&mut terminal)?;
    println!("terminal too small, must be at least 74x21");
    return Ok(());
  }

  let tick_rate = Duration::from_millis(20);
  let eye = Eye::new(size);
  let res = run(&mut terminal, eye, tick_rate);

  quit(&mut terminal)?;

  if let Err(err) = res {
    println!("{err:?}");
  }
  Ok(())
}

fn quit<B>(terminal: &mut Terminal<B>) -> anyhow::Result<()>
where
  B: Backend + io::Write,
{
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
  terminal.show_cursor()?;
  Ok(())
}

fn run<B: Backend>(
  terminal: &mut Terminal<B>, mut eye: Eye, tick_rate: Duration,
) -> anyhow::Result<()> {
  let mut last_tick = Instant::now();
  for m in 1.. {
    terminal.draw(|f| ui(f, &eye))?;
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
      eye.tick(m)?;
      last_tick = Instant::now();
    }
  }
  Ok(())
}

struct Eye {
  state: String,
  height: u16,
  width: u16,
  v: Box<dyn Fn(f64, f64, f64) -> usize>,
}

impl Eye {
  fn new(size: Rect) -> Self {
    let v = move |x: f64, t: f64, s: f64| {
      let a = (x - C).abs();
      let j = (t - Q).abs();
      if j > P || a > 2.0 * G {
        0
      } else {
        (s * -(2.0 * P * G - P * a - G * j) + s) as usize
      }
    };
    Self {
      v: Box::new(v),
      state: String::default(),
      height: size.height,
      width: size.width,
    }
  }

  fn tick(&mut self, m: usize) -> anyhow::Result<()> {
    let mut s = b"\n  ".to_vec();
    for t in 0..self.height {
      for x in 0..self.width {
        let d = (self.v)(
          x as f64 / self.width as f64 * 3.0,
          t as f64 / (self.height as f64 - C),
          m as f64,
        );
        s.push(b" .:-=+*&#%@"[d % 11]);
      }
      s.extend_from_slice(b"\n  ");
    }
    self.state = String::from_utf8(s)?;
    Ok(())
  }
}

fn ui<B: Backend>(f: &mut Frame<B>, eye: &Eye) {
  let label = Paragraph::new(eye.state.clone());
  // let block = Block::default().title("").borders(Borders::ALL);
  f.render_widget(label, f.size());
}
