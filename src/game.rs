use std::io;

use thiserror::Error;

const SIDE_LENGTH: u8 = 3;
const BOARD_SIZE: u8 = SIDE_LENGTH * SIDE_LENGTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSquare {
    X,
    O,
}

impl GameSquare {
    pub fn name(self) -> &'static str {
        match self {
            GameSquare::X => "X",
            GameSquare::O => "O",
        }
    }
}

pub struct GameBoard([Option<GameSquare>; BOARD_SIZE as usize]);

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("Index was too big. Pick a number from 1 through {BOARD_SIZE}.")]
    PosTooBig(u8),
    #[error("{} had already moved there.", .0.name())]
    AlreadyMoved(GameSquare),
}
impl GameBoard {
    pub fn new() -> Self {
        GameBoard([None; 3 * 3])
    }

    pub fn encode_to(&self, sink: &mut impl io::Write) {
        for (i, square) in self.0.iter().enumerate() {
            let index_char = [(i + 1) as u8 + b'0'];
            sink.write_all(square.map(|s| s.name().as_bytes()).unwrap_or(&index_char))
                .unwrap();
            // Put newlines after every 3.
            if (i + 1) % 3 == 0 {
                sink.write(b"\n").unwrap();
            }
        }
    }

    /// `pos` is 1-indexed.
    pub fn make_move(&mut self, player: GameSquare, pos: u8) -> Result<bool, MoveError> {
        let pos = pos - 1;
        if pos >= BOARD_SIZE {
            Err(MoveError::PosTooBig(pos))
        } else if let Some(p) = self.0[pos as usize] {
            Err(MoveError::AlreadyMoved(p))
        } else {
            self.0[pos as usize] = Some(player);
            let winner = self.winner();
            if winner != None && winner != Some(player) {
                panic!("Made a move and the other guy won.");
            }
            Ok(winner == Some(player))
        }
    }

    fn winner(&self) -> Option<GameSquare> {
        fn check_pattern(
            pattern: impl Iterator<Item = impl Iterator<Item = Option<GameSquare>>>,
        ) -> Option<GameSquare> {
            for mut series in pattern {
                let first = series.next().unwrap();
                if first == None {
                    continue;
                }
                if series.all(|s| s == first) {
                    return first;
                }
            }
            None
        }

        let rows = self
            .0
            .chunks_exact(SIDE_LENGTH as usize)
            .map(|s| s.iter().copied());
        let columns =
            (0..SIDE_LENGTH).map(|y| (0..SIDE_LENGTH).map(move |x| self.0[(3 * x + y) as usize]));

        check_pattern(rows)
            .or_else(|| check_pattern(columns))
            .or_else(|| None /* TODO: Handle diagonals. */)
    }
}
