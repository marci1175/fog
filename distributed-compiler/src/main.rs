use std::{io::stdout, sync::Arc, time::Duration};

use color_eyre::{Result, eyre::Context};
use common::{dotenvy, tokio};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use distributed_compiler::{UiState, io::ServerState, worker::ThreadIdentification};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{self, Borders, List, ListItem, Paragraph},
};

#[derive(Debug, Clone)]
struct App
{
    ui_state: UiState,
    selected: usize,
    should_quit: bool,

    // Client scrolling
    scroll: u16,
    max_scroll: u16,

    // Dedicated log viewer
    thread_logs: Vec<(String, ThreadIdentification)>,
    log_scroll: u16,
    log_max_scroll: u16,

    // .env
    env_port: Option<u16>,
    dep_man_url: Option<String>,
    dep_folder: Option<String>,
}

impl App
{
    fn new() -> Self
    {
        let env_port = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok());

        let dep_man_url = std::env::var("DEPENDENCY_MANAGER_URL").ok();
        let dep_folder = std::env::var("DEPENDENCY_FOLDER").ok();

        Self {
            ui_state: UiState::Main,
            selected: 0,
            should_quit: false,

            scroll: 0,
            max_scroll: 0,

            thread_logs: Vec::new(),
            log_scroll: 0,
            log_max_scroll: 0,

            env_port,
            dep_man_url,
            dep_folder,
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

        match &mut self.ui_state {
            UiState::Main => self.key_main(key),
            UiState::ConnectionEstablisher => self.key_connection_establisher(key),
            UiState::CurrentConnection(_) => self.key_current_connection(key),
            UiState::LogViewer(_) => self.key_log_viewer(key),
        }
    }

    fn key_main(&mut self, key: KeyEvent)
    {
        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            },
            KeyCode::Down => {
                if self.selected + 1 < self.menu_items().len() {
                    self.selected += 1;
                }
            },
            KeyCode::Enter => {
                match self.selected {
                    0 => self.ui_state = UiState::ConnectionEstablisher,
                    1 => {},
                    2 => self.should_quit = true,
                    _ => {},
                }
            },
            KeyCode::Esc => self.should_quit = true,
            _ => {},
        }
    }

    fn key_connection_establisher(&mut self, key: KeyEvent)
    {
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

                let dependency_folder = self.dep_folder.clone().unwrap_or(String::from("deps/"));

                let mut server_state = ServerState::new(
                    port,
                    dependency_manager_url,
                    Arc::new(dependency_folder.into()),
                );

                server_state.initialize_server().unwrap();

                self.ui_state = UiState::CurrentConnection(server_state);
            },
            _ => {},
        }
    }

    fn key_current_connection(&mut self, key: KeyEvent)
    {
        match key.code {
            KeyCode::Char('l') | KeyCode::Char('L') => {
                // switch UI to log viewer
                if let UiState::CurrentConnection(ss) = &self.ui_state {
                    self.ui_state = UiState::LogViewer(ss.clone());
                }
                return;
            },

            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = (self.scroll + 1).min(self.max_scroll),

            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
            },
            KeyCode::PageDown => {
                self.scroll = (self.scroll + 10).min(self.max_scroll);
            },

            KeyCode::Esc => {},
            _ => {},
        }
    }

    fn key_log_viewer(&mut self, key: KeyEvent)
    {
        match key.code {
            KeyCode::Esc => {
                // go back to current connection
                if let UiState::LogViewer(ss) = &self.ui_state {
                    self.ui_state = UiState::CurrentConnection(ss.clone());
                }
            },

            KeyCode::Up => {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            },
            KeyCode::Down => {
                self.log_scroll = (self.log_scroll + 1).min(self.log_max_scroll);
            },

            KeyCode::PageUp => {
                self.log_scroll = self.log_scroll.saturating_sub(10);
            },

            KeyCode::PageDown => {
                self.log_scroll = (self.log_scroll + 10).min(self.log_max_scroll);
            },

            _ => {},
        }
    }
}

#[tokio::main]
async fn main() -> Result<()>
{
    color_eyre::install()?;

    dotenvy::dotenv().ok();

    enable_raw_mode().context("failed to enable raw mode")?;
    let mut terminal = DefaultTerminal::new(CrosstermBackend::new(stdout()))?;

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
        UiState::LogViewer(ss) => render_log_viewer(frame, app, &ss),
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
        "FDCN offloads compilation work to remote workers. Press Enter to continue.",
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

    let list = List::new(items).highlight_style(Style::default().bg(Color::White).fg(Color::Black));

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
    // Drain logs
    if let Some(rx) = &server_state.thread_error_out {
        while let Ok((msg, id)) = rx.try_recv() {
            app.thread_logs.push((msg, id));
            if app.thread_logs.len() > 500 {
                app.thread_logs.remove(0);
            }
        }
    }

    let area = frame.area();

    let block = widgets::Block::new()
        .title("Connected Clients — Press L for Logs")
        .title_alignment(Alignment::Center)
        .borders(Borders::all());
    frame.render_widget(&block, area);

    let inner = block.inner(area);

    let mut rows = Vec::new();

    for entry in server_state.connected_clients.iter() {
        let addr = entry.key();
        let info = entry.value();

        rows.push(widgets::Row::new(vec![
            addr.to_string(),
            format!("{}", info),
        ]));
    }

    // table scrolling
    let height = inner.height.saturating_sub(2);
    let total = rows.len() as u16;

    app.max_scroll = total.saturating_sub(height);
    if app.scroll > app.max_scroll {
        app.scroll = app.max_scroll;
    }

    let start = app.scroll as usize;
    let end = (start + height as usize).min(rows.len());
    let visible = rows[start..end].to_vec();

    let table = widgets::Table::new(
        visible,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(
        widgets::Row::new(vec!["Client", "Information"])
            .style(Style::default().add_modifier(ratatui::style::Modifier::BOLD)),
    )
    .block(widgets::Block::new().borders(Borders::all()))
    .column_spacing(2);

    frame.render_widget(table, inner);
}

fn render_log_viewer(frame: &mut Frame, app: &mut App, _ss: &ServerState)
{
    let area = frame.area();

    let block = widgets::Block::new()
        .title("Thread Log Viewer — Esc to return")
        .title_alignment(Alignment::Center)
        .borders(Borders::all());
    frame.render_widget(&block, area);

    let inner = block.inner(area);

    // Compute log scroll
    let height = inner.height.saturating_sub(2);
    let total_logs = app.thread_logs.len() as u16;

    app.log_max_scroll = total_logs.saturating_sub(height);
    if app.log_scroll > app.log_max_scroll {
        app.log_scroll = app.log_max_scroll;
    }

    let start = app.log_scroll as usize;
    let end = (start + height as usize).min(app.thread_logs.len());

    let lines: Vec<Line> = app.thread_logs[start..end]
        .iter()
        .map(|(msg, tid)| Line::from(Span::raw(format!("T{}: {}", tid.id, msg))))
        .collect();

    let paragraph = Paragraph::new(Text::from(lines))
        .block(widgets::Block::new().borders(Borders::all()))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, inner);
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
