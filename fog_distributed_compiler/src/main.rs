use std::{io::stdout, time::Duration};

use color_eyre::{Result, eyre::Context};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use fog_common::tokio;
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

    fn key_handler(
        &mut self,
        key: KeyEvent,
        port_field_state: &TextField,
        terminal: &mut DefaultTerminal,
    )
    {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Enter => {
                terminal.clear().unwrap();
                self.selected = 0;
            },

            _ => (),
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
            },
            UiState::ConnectionEstablisher => {
                match key.code {
                    KeyCode::Esc => {
                        self.ui_state = UiState::Main;
                    },
                    KeyCode::Enter => {
                        // Parse text input dont display an error and only switch to current connection if the port is valid
                        if let Ok(port) = port_field_state.inner_text.parse::<u32>() {
                            let mut server_state = ServerState::new(port);

                            server_state.initialize_server();

                            self.ui_state = UiState::CurrentConnection(server_state);
                        }
                    },
                    _ => {},
                }
            },
            UiState::CurrentConnection(_) => {
                match key.code {
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

    let mut port_field = TextField::new(
        "*Enter Port*",
        Style::default(),
        Style::new().bg(Color::White).fg(Color::Black),
    );

    while !app.should_quit.clone() {
        if let Some(key_event) = capture_input()? {
            app.key_handler(key_event, &port_field, terminal);
        }

        terminal.draw(|f| render(f, &app, &mut port_field))?;
    }

    terminal.clear()?;

    Ok(())
}

/// Render whole UI based on App state (pure)
fn render(frame: &mut Frame, app: &App, port_field: &mut TextField)
{
    match &app.ui_state {
        UiState::Main => render_main(frame, app),
        UiState::ConnectionEstablisher => render_connection_establisher(frame, app, port_field),
        UiState::CurrentConnection(ss) => render_current_connection(frame, ss.clone()),
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
        "FDCN is a build accelerator tool created to lower build times by offloading compilation jobs to remote worker(s). (CLI key input may be buggy for windows users.)",
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

fn render_connection_establisher(frame: &mut Frame, app: &App, port_field: &mut TextField)
{
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
            },
            KeyCode::Backspace => {
                port_field.inner_text.pop();
            },
            _ => (),
        }
    }

    port_field.should_highlight(app.selected == 0);

    frame.render_widget(port_field.clone(), chunks[1]);
}

use ratatui::widgets::{Block, Row, Table};

fn render_current_connection(frame: &mut Frame, server_state: ServerState)
{
    let area = frame.area();
    let block = widgets::Block::new()
        .title(format!(
            "Port: {} | Currently Connected: {}",
            server_state.port,
            server_state.connected_clients.len()
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::all());

    frame.render_widget(&block, area);
    let inner = block.inner(area);

    let clients: Vec<(String, String)> = server_state
        .connected_clients
        .iter()
        .map(|entry| {
            let ip = entry.key().to_string();
            let name = entry.value().to_string();
            (ip, name)
        })
        .collect();

    if clients.is_empty() {
        let text = Paragraph::new("No clients connected.").alignment(Alignment::Center);
        frame.render_widget(text, inner);
        return;
    }

    let cols = 3; // Adjust number of columns here if you want
    let widths = vec![Constraint::Percentage(50), Constraint::Percentage(50)];

    // Convert clients into Table Rows
    let rows = clients.chunks(cols).map(|chunk| {
        Row::new(
            chunk
                .iter()
                .map(|(ip, name)| Cell::from(format!("{} ({})", name, ip))),
        )
    });

    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::ALL).title("Clients"))
        .column_spacing(1);

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

            // Accept only the FIRST press, ignore press repeats
            Ok(Some(key))
        },
        _ => Ok(None),
    }
}
