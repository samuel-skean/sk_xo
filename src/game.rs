use std::io;

#[derive(Debug, Clone, Copy)]
pub enum GameSquare {
    X,
    O,
}

pub struct GameBoard([Option<GameSquare>; 3 * 3]);

impl GameBoard {
    pub fn new() -> Self {
        GameBoard([None; 3 * 3])
    }

    pub fn encode_to(&self, sink: &mut impl io::Write) {
        for (i, square) in self.0.iter().enumerate() {
            let index_char = [(i + 1) as u8 + b'0'];
            sink.write_all(match square {
                Some(GameSquare::X) => b"X",
                Some(GameSquare::O) => b"O",
                None => &index_char,
            })
            .unwrap();
            // Put newlines after every 3.
            if (i + 1) % 3 == 0 {
                sink.write(b"\n").unwrap();
            }
        }
    }
}
