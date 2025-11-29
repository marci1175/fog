use std::{io::stdout, time::Duration};

use color_eyre::{Result, eyre::Context};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use fog_distributed_compiler::UiState;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Text,
    widgets::{self, Borders, List, ListItem, Paragraph},
};

/// Simple app state
struct App
{
    ui_state: UiState,
    selected: usize,
    should_quit: bool,
}

impl App
{
    fn new() -> Self
    {
        Self {
            ui_state: UiState::Main,
            selected: 0,
            should_quit: false,
        }
    }

    fn menu_items(&self) -> [&'static str; 3]
    {
        ["Start Compiler Server", "Help", "Quit"]
    }

    /// Handle a key event (update phase)
    fn on_key(&mut self, key: KeyEvent)
    {
        match self.ui_state {
            UiState::Main => {
                self.on_key_main(key);
            },
            UiState::ConnectionEstablisher => {
                // later: handle keys for this screen
                if key.code == KeyCode::Esc {
                    self.ui_state = UiState::Main;
                }
            },
            UiState::CurrentConnection => {
                // later: handle keys here too
                if key.code == KeyCode::Esc {
                    self.ui_state = UiState::Main;
                }
            },
        }
    }

    fn on_key_main(&mut self, key: KeyEvent)
    {
        let item_count = self.menu_items().len();

        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            },
            KeyCode::Down => {
                if self.selected + 1 < item_count {
                    self.selected += 1;
                }
            },
            KeyCode::Enter => {
                // Run the selected "callback" here (update logic)
                match self.selected {
                    0 => {
                        // Start Compiler Server
                        self.ui_state = UiState::ConnectionEstablisher;
                    },
                    1 => {
                        // Help
                    },
                    2 => {
                        // Quit
                        self.should_quit = true;
                    },
                    _ => {},
                }
            },
            KeyCode::Esc => {
                self.should_quit = true;
            },
            _ => {},
        }
    }
}

fn main() -> Result<()>
{
    color_eyre::install()?;

    // Set up terminal
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut terminal = DefaultTerminal::new(CrosstermBackend::new(stdout()))
        .context("failed to create terminal")?;

    let res = run(&mut terminal);

    // Cleanup
    disable_raw_mode().context("failed to disable raw mode")?;
    terminal.show_cursor().context("failed to show cursor")?;

    res
}

fn run(terminal: &mut DefaultTerminal) -> Result<()>
{
    let mut app = App::new();
    terminal.clear()?;

    while !app.should_quit {
        if let Some(key_event) = capture_input()? {
            app.on_key(key_event);
        }

        // 2. Render phase (pure)
        terminal.draw(|f| render(f, &app))?;
    }

    terminal.flush()?;
    
    Ok(())
}

/// Render whole UI based on App state (pure)
fn render(frame: &mut Frame, app: &App)
{
    match app.ui_state {
        UiState::Main => render_main(frame, app),
        UiState::ConnectionEstablisher => render_connection_establisher(frame),
        UiState::CurrentConnection => render_current_connection(frame),
    }
}

fn render_main(frame: &mut Frame, app: &App)
{
    let area = frame.area();

    let block = widgets::Block::new()
        .title("Fog Distributed Compiler Network [FDCN]")
        .title_alignment(Alignment::Center)
        .borders(Borders::all());

    frame.render_widget(&block, area);
    let inner = block.inner(area);

    // Vertical layout: [description] [menu]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(3)].as_ref())
        .split(inner);

    let cli_desc = Paragraph::new(
        "FDCN is a build accelerator tool created to lower build times by \
         offloading compilation jobs to remote worker(s).",
    )
    .alignment(Alignment::Center);

    frame.render_widget(cli_desc, chunks[0]);

    // Menu list
    let items: Vec<ListItem> = app
        .menu_items()
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let text = if idx == app.selected {
                Text::from(*label).patch_style(Style::default().bg(Color::White).fg(Color::Black))
            }
            else {
                Text::from(*label)
            };
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(widgets::Block::default())
        .highlight_style(Style::default().bg(Color::White).fg(Color::Black));

    frame.render_widget(list, chunks[1]);
}

fn render_connection_establisher(frame: &mut Frame)
{
    let area = frame.area();
    let block = widgets::Block::new()
        .title("Connection Establisher")
        .title_alignment(Alignment::Center)
        .borders(Borders::all());
    frame.render_widget(&block, area);

    let inner = block.inner(area);
    let text = Paragraph::new("TODO: implement connection establisher UI.\nPress Esc to go back.")
        .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

fn render_current_connection(frame: &mut Frame)
{
    let area = frame.area();
    let block = widgets::Block::new()
        .title("Current Connection")
        .title_alignment(Alignment::Center)
        .borders(Borders::all());
    frame.render_widget(&block, area);

    let inner = block.inner(area);
    let text = Paragraph::new("TODO: implement current connection UI.\nPress Esc to go back.")
        .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Read a key press (non-blocking-ish with timeout)
fn capture_input() -> Result<Option<KeyEvent>>
{
    if !event::poll(Duration::from_millis(16))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            // Accept only the FIRST press, ignore press repeats
            Ok(Some(key))
        }
        _ => Ok(None),
    }
}
