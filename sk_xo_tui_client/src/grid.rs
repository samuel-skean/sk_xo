use ratatui::prelude::*;

pub fn text_from_squares(squares: Vec<&str>, x: usize, y: usize) -> Text<'static> {
    // TODO: Real error handling.
    assert_eq!(squares.len(), x * y);

    let rows: Vec<_> = squares.chunks(x).map(|row| row.join(" ")).collect();

    Text::from(rows.join("\n\n"))
}
