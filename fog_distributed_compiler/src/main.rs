use std::{io::stdout, time::Duration};

use color_eyre::{Result, eyre::Context};
use crossterm::{
    event::{self, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{DefaultTerminal, Frame, prelude::CrosstermBackend, widgets::Paragraph};

fn main() -> Result<()>
{
    color_eyre::install()?;

    // Set up terminal
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut terminal = DefaultTerminal::new(CrosstermBackend::new(stdout()))
        .context("failed to create terminal")?;

    // Run app loop
    let res = run(&mut terminal);

    // Cleanup
    disable_raw_mode().context("failed to disable raw mode")?;
    terminal.show_cursor().context("failed to show cursor")?;

    res
}

fn run(terminal: &mut DefaultTerminal) -> Result<()>
{
    terminal.clear()?;

    loop {
        terminal.draw(render)?;

        if should_quit()? {
            break;
        }
    }
    Ok(())
}

fn render(frame: &mut Frame)
{
    let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    frame.render_widget(greeting, frame.area());
}

/// Returns true if 'q' is pressed
fn should_quit() -> Result<bool>
{
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        let q_pressed = event::read()
            .context("event read failed")?
            .as_key_press_event()
            .is_some_and(|key| key.code == KeyCode::Char('q'));
        return Ok(q_pressed);
    }
    Ok(false)
}
