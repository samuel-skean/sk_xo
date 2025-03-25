use std::{fs, os::unix::net::UnixDatagram, path::Path, time::Duration};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    DefaultTerminal,
    prelude::*,
    widgets::{Block, Paragraph, Widget},
};

/// Set a panic hook that deletes the specified path before calling the original
/// panic hook.
/// 
/// I'm doing this instead of relying on Drop because Drop may not be called on
/// panics.
fn delete_file_on_panic(path: &'static Path) {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Err(e) = fs::remove_file(path) {
            eprintln!(
                "Encountered error when removing file '{}': {e}",
                path.to_string_lossy()
            );
        }
        old_hook(info);
    }));
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let socket = UnixDatagram::bind("client_test.sock")?;
    delete_file_on_panic("client_test.sock".as_ref());
    let mut terminal = ratatui::init();
    let result = App::new(socket).run(&mut terminal);
    ratatui::restore();
    fs::remove_file("client_test.sock").unwrap();
    result
}

#[derive(Debug)]
struct App {
    socket: UnixDatagram,
    counter: u8,
    exit: bool,
}

impl App {
    fn new(socket: UnixDatagram) -> Self {
        socket.set_nonblocking(true).unwrap();
        App {
            socket,
            counter: 0,
            exit: false,
        }
    }
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        event::KeyCode::Left => {
                            self.counter = self.counter.saturating_sub(1);
                        }
                        event::KeyCode::Right => {
                            self.counter = self.counter.saturating_add(1);
                        }
                        event::KeyCode::Char('q') => {
                            self.exit = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Process a packet after blocking, so it's one step closer to the
        // screen refresh.
        let mut buf = [0; 0x1000];
        if let Ok(dgram_size) = self.socket.recv(&mut buf) {
            let change = std::str::from_utf8(&buf[..dgram_size])
                .unwrap()
                .parse()
                .unwrap();
            self.counter = self.counter.saturating_add(change);
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Counter App".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".into(),
            " Increment ".into(),
            "<Right>".into(),
            " Quit ".into(),
            "<Q>".into(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered());

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);
        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
