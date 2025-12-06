use std::{io::stdout, time::Duration};

use color_eyre::{Result, eyre::Context};
use common::{dotenvy, tokio};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use fog_distributed_compiler::{UiState, io::ServerState};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Text,
    widgets::{self, Borders, List, ListItem, Paragraph},
};

#[derive(Debug, Clone)]
struct App
{
    ui_state: UiState,
    selected: usize,
    should_quit: bool,

    scroll: u16,
    max_scroll: u16,

    env_port: Option<u16>,
    dep_man_url: Option<String>,
}

impl App
{
    fn new() -> Self
    {
        // Load PORT from .env at startup
        let env_port = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok());

        let dep_man_url = std::env::var("DEPENDENCY_MANAGER_URL").ok();

        Self {
            ui_state: UiState::Main,
            selected: 0,
            should_quit: false,

            scroll: 0,
            max_scroll: 0,

            env_port,
            dep_man_url,
        }
    }

    fn menu_items(&self) -> [&'static str; 3]
    {
        ["Start Compiler Server", "Help", "Quit"]
    }

    fn key_handler(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal)
    {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match self.ui_state {
            UiState::Main => {
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
                        match self.selected {
                            0 => {
                                self.ui_state = UiState::ConnectionEstablisher;
                            },
                            1 => {}, // help page later
                            2 => {
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
            },

            UiState::ConnectionEstablisher => {
                match key.code {
                    KeyCode::Esc => {
                        self.ui_state = UiState::Main;
                    },
                    KeyCode::Enter => {
                        let port = self.env_port.unwrap_or(3004);
                        let dependency_manager_url = self
                            .dep_man_url
                            .clone()
                            .unwrap_or(String::from("http://[::1]:3004"));

                        let mut server_state = ServerState::new(port, dependency_manager_url);
                        server_state.initialize_server().unwrap();
                        self.ui_state = UiState::CurrentConnection(server_state);
                    },
                    _ => {},
                }
            },

            UiState::CurrentConnection(_) => {
                match key.code {
                    KeyCode::Up => {
                        if self.scroll > 0 {
                            self.scroll -= 1;
                        }
                    },
                    KeyCode::Down => {
                        if self.scroll < self.max_scroll {
                            self.scroll += 1;
                        }
                    },
                    KeyCode::PageUp => {
                        self.scroll = self.scroll.saturating_sub(10);
                    },
                    KeyCode::PageDown => {
                        self.scroll = (self.scroll + 10).min(self.max_scroll);
                    },
                    KeyCode::Esc => {},
                    _ => {},
                }
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()>
{
    color_eyre::install()?;

    // Load .env before starting UI
    dotenvy::dotenv().ok();

    enable_raw_mode().context("failed to enable raw mode")?;
    let mut terminal = DefaultTerminal::new(CrosstermBackend::new(stdout()))
        .context("failed to create terminal")?;

    let res = run(&mut terminal);

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
            app.key_handler(key_event, terminal);
        }

        terminal.draw(|f| render(f, &mut app))?;
    }

    terminal.clear()?;
    Ok(())
}

fn render(frame: &mut Frame, app: &mut App)
{
    match app.ui_state.clone() {
        UiState::Main => render_main(frame, app),
        UiState::ConnectionEstablisher => render_connection_establisher(frame, app),
        UiState::CurrentConnection(ss) => render_current_connection(frame, app, &ss),
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(3)].as_ref())
        .split(inner);

    let cli_desc = Paragraph::new(
        "FDCN offloads compilation work to remote workers. (Windows input delay may occur.)",
    )
    .alignment(Alignment::Center);

    frame.render_widget(cli_desc, chunks[0]);

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

fn render_connection_establisher(frame: &mut Frame, app: &App)
{
    let area = frame.area();

    let block = widgets::Block::new()
        .title("Connection Establisher | Press Esc to go back.")
        .title_alignment(Alignment::Center)
        .borders(Borders::all());

    frame.render_widget(&block, area);
    let inner = block.inner(area);

    let port = app
        .env_port
        .map(|v| v.to_string())
        .unwrap_or_else(|| "NOT SET".into());

    let dep_man_url: String = app.dep_man_url.clone().unwrap_or_else(|| "NOT SET".into());

    let text = Paragraph::new(format!(
        "Server will start using PORT from .env\n\nConfigured PORT: {port}\n\nConfigured Dependency Manager: {dep_man_url}\n\nPress Enter to start.",
    ))
    .alignment(Alignment::Center);

    frame.render_widget(text, inner);
}

fn render_current_connection(frame: &mut Frame, app: &mut App, server_state: &ServerState)
{
    let area = frame.area();

    let block = widgets::Block::new()
        .title(format!(
            "Port: {} | Connected: {} | Scroll: {} / {}",
            server_state.port,
            server_state.connected_clients.len(),
            app.scroll,
            app.max_scroll,
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::all());

    frame.render_widget(&block, area);
    let inner = block.inner(area);

    // Collect all rows
    let mut rows = Vec::new();

    for entry in server_state.connected_clients.iter() {
        let socket_addr = entry.key();
        let info = entry.value();

        rows.push(widgets::Row::new(vec![
            format!("Client {}", socket_addr),
            format!("{}", info.len()),
            "".into(),
        ]));
    }

    // How many rows fit on screen?
    let visible_height = inner.height.saturating_sub(2); // header + maybe some padding
    let total_rows = rows.len() as u16;

    // Compute and store max_scroll
    let max_scroll = total_rows.saturating_sub(visible_height);
    app.max_scroll = max_scroll;

    // Clamp scroll in case window was resized / rows changed
    if app.scroll > app.max_scroll {
        app.scroll = app.max_scroll;
    }

    // Slice rows to what’s visible
    let start = app.scroll as usize;
    let end = (start + visible_height as usize).min(rows.len());
    let visible_rows = rows[start..end].to_vec();

    let table = widgets::Table::new(
        visible_rows,
        [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ],
    )
    .header(
        widgets::Row::new(vec!["Thread ID", "Jobs", "Output"])
            .style(Style::default().add_modifier(ratatui::style::Modifier::BOLD)),
    )
    .block(
        widgets::Block::new()
            .borders(Borders::all())
            .title("Worker Threads")
            .title_alignment(Alignment::Center),
    )
    .column_spacing(2);

    frame.render_widget(table, inner);
}

fn capture_input() -> Result<Option<KeyEvent>>
{
    if !event::poll(Duration::from_millis(100))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key) => {
            if (key.code == KeyCode::Up || key.code == KeyCode::Down || key.code == KeyCode::Enter)
                && key.kind != KeyEventKind::Press
            {
                return Ok(None);
            }
            Ok(Some(key))
        },
        _ => Ok(None),
    }
}
