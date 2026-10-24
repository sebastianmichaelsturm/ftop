pub mod app;
pub mod filter;
pub mod model;
pub mod ui;
pub mod watcher;

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    fs,
    io::{self, stdout},
    path::PathBuf,
    time::Duration,
};
use watcher::FileWatcher;

#[derive(Parser, Debug)]
#[command(
    name = "ftop",
    about = "A real-time top-like terminal directory explorer tracking filesystem changes",
    version = "0.1.0"
)]
pub struct Args {
    /// Directory to watch (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Show all files, including noisy directories like .git and node_modules
    #[arg(short, long, default_value_t = false)]
    pub all: bool,
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let root_path = fs::canonicalize(&args.path)?;

    // Set panic hook to ensure terminal resets even if an unexpected error occurs
    setup_panic_hook();

    // Initialize watcher
    let watcher = FileWatcher::new(&root_path)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(root_path, args.all);

    let res = run_loop(&mut terminal, &mut app, &watcher);

    // Teardown terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err}");
    }

    Ok(())
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    watcher: &FileWatcher,
) -> io::Result<()> {
    loop {
        // Drain any pending filesystem events
        while let Ok(event_res) = watcher.rx.try_recv() {
            if let Ok(event) = event_res {
                app.process_event(event);
            }
        }

        // Render current state
        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for user keyboard events at ~30 FPS (33ms)
        if event::poll(Duration::from_millis(33))? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.is_filtering {
                        match key.code {
                            KeyCode::Esc => {
                                app.is_filtering = false;
                                app.filter_query.clear();
                            }
                            KeyCode::Enter => {
                                app.is_filtering = false;
                            }
                            KeyCode::Backspace => {
                                app.filter_query.pop();
                            }
                            KeyCode::Char(c) => {
                                app.filter_query.push(c);
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc => {
                                if !app.filter_query.is_empty() {
                                    app.filter_query.clear();
                                } else if app.is_paused {
                                    app.toggle_pause();
                                } else {
                                    return Ok(());
                                }
                            }
                            KeyCode::Char(' ') => app.toggle_pause(),
                            KeyCode::Char('/') => app.is_filtering = true,
                            KeyCode::Enter => app.open_selected(),
                            KeyCode::Char('s') => app.cycle_sort(),
                            KeyCode::Char('c') => app.clear(),
                            KeyCode::Down | KeyCode::Char('j') => app.next_row(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous_row(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
