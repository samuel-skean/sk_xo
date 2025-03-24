use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    DefaultTerminal,
    prelude::*,
    widgets::{Block, Paragraph, Widget},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Debug, Default)]
struct App {
    counter: u8,
    exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    event::KeyCode::Left => { self.counter = self.counter.saturating_sub(1); }
                    event::KeyCode::Right => { self.counter = self.counter.saturating_add(1); }
                    event::KeyCode::Char('q') => { self.exit = true; },
                    _ => {}
                }
            }
            _ => {}
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
        let block = Block::bordered().title(title.centered()).title_bottom(instructions.centered());

        let counter_text = Text::from(vec![
            Line::from(vec![
                "Value: ".into(),
                self.counter.to_string().yellow(),
            ])
        ]);
        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
