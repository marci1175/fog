use std::{io::stdout, net::SocketAddr, time::Duration};

use color_eyre::{Result, eyre::Context};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use common::tokio;
use fog_distributed_compiler::{TextField, UiState, io::ServerState};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Text,
    widgets::{self, Borders, Cell, List, ListItem, Paragraph},
};

#[derive(Debug, Clone)]
struct App {
    ui_state: UiState,
    selected: usize,
    should_quit: bool,

    scroll: u16,
    max_scroll: u16,
}

impl App {
    fn new() -> Self {
        Self {
            ui_state: UiState::Main,
            selected: 0,
            should_quit: false,

            scroll: 0,
            max_scroll: 0,
        }
    }

    fn menu_items(&self) -> [&'static str; 3] {
        ["Start Compiler Server", "Help", "Quit"]
    }

    fn key_handler(
        &mut self,
        key: KeyEvent,
        port_field_state: &TextField,
        terminal: &mut DefaultTerminal,
    ) {
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
                    }
                    KeyCode::Down => {
                        if self.selected + 1 < item_count {
                            self.selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        match self.selected {
                            0 => {
                                self.ui_state = UiState::ConnectionEstablisher;
                            }
                            1 => {
                                // help page later
                            }
                            2 => {
                                self.should_quit = true;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }

            UiState::ConnectionEstablisher => {
                match key.code {
                    KeyCode::Esc => {
                        self.ui_state = UiState::Main;
                    }
                    KeyCode::Enter => {
                        if let Ok(port) = port_field_state.inner_text.parse::<u16>() {
                            let mut server_state = ServerState::new(port);
                            server_state.initialize_server().unwrap();
                            self.ui_state = UiState::CurrentConnection(server_state);
                        }
                    }
                    _ => {}
                }
            }

            UiState::CurrentConnection(_) => {
                match key.code {
                    KeyCode::Up => {
                        if self.scroll > 0 {
                            self.scroll -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.scroll < self.max_scroll {
                            self.scroll += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        self.scroll = self.scroll.saturating_sub(10);
                    }
                    KeyCode::PageDown => {
                        self.scroll = (self.scroll + 10).min(self.max_scroll);
                    }
                    KeyCode::Esc => {
                        // could add back navigation later
                    }
                    _ => {}
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    enable_raw_mode().context("failed to enable raw mode")?;
    let mut terminal = DefaultTerminal::new(CrosstermBackend::new(stdout()))
        .context("failed to create terminal")?;

    let res = run(&mut terminal);

    disable_raw_mode().context("failed to disable raw mode")?;
    terminal.show_cursor().context("failed to show cursor")?;

    res
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    terminal.clear()?;

    let mut port_field = TextField::new(
        "*Enter Port*",
        Style::default(),
        Style::new().bg(Color::White).fg(Color::Black),
    );

    while !app.should_quit.clone() {
        if let Some(key_event) = capture_input()? {
            app.key_handler(key_event, &port_field, terminal);
        }

        terminal.draw(|f| render(f, &mut app, &mut port_field))?;
    }

    terminal.clear()?;
    Ok(())
}

fn render(frame: &mut Frame, app: &mut App, port_field: &mut TextField) {
    match app.ui_state.clone() {
        UiState::Main => render_main(frame, app),
        UiState::ConnectionEstablisher => render_connection_establisher(frame, app, port_field),
        UiState::CurrentConnection(ss) => render_current_connection(frame, app, &ss),
    }
}

fn render_main(frame: &mut Frame, app: &App) {
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
            } else {
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

fn render_connection_establisher(frame: &mut Frame, app: &App, port_field: &mut TextField) {
    let area = frame.area();
    let block = widgets::Block::new()
        .title("Connection Establisher | Press Esc to go back.")
        .title_alignment(Alignment::Center)
        .borders(Borders::all());

    frame.render_widget(&block, area);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3)])
        .split(inner);

    let text = Text::raw("Open server on port:");
    frame.render_widget(text, chunks[0]);

    if let Ok(Some(key_event)) = capture_input() {
        match key_event.code {
            KeyCode::Char(char) => {
                if char.is_numeric() {
                    port_field.inner_text.push(char);
                }
            }
            KeyCode::Backspace => {
                port_field.inner_text.pop();
            }
            _ => (),
        }
    }

    port_field.should_highlight(app.selected == 0);
    frame.render_widget(port_field.clone(), chunks[1]);
}

fn render_current_connection(frame: &mut Frame, app: &mut App, server_state: &ServerState) {
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

fn capture_input() -> Result<Option<KeyEvent>> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key) => {
            if (key.code == KeyCode::Up
                || key.code == KeyCode::Down
                || key.code == KeyCode::Enter)
                && key.kind != KeyEventKind::Press
            {
                return Ok(None);
            }
            Ok(Some(key))
        }
        _ => Ok(None),
    }
}
