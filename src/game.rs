use std::fmt;

use thiserror::Error;

const SIDE_LENGTH: u8 = 3;
const BOARD_SIZE: u8 = SIDE_LENGTH * SIDE_LENGTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSquare {
    X,
    O,
}

impl fmt::Display for GameSquare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            GameSquare::X => "X",
            GameSquare::O => "O",
        })
    }
}

pub struct GameBoard([Option<GameSquare>; BOARD_SIZE as usize]);

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("Index was too big. Pick a number from 1 through {BOARD_SIZE}.")]
    PosTooBig(u8),
    #[error("{0} had already moved there.")]
    AlreadyMoved(GameSquare),
}

impl fmt::Display for GameBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, square) in self.0.iter().enumerate() {
            if let Some(filled_square) = square {
                write!(f, "{filled_square}")?;
            } else {
                write!(f, "{}", i + 1)?;
            }
            // Put newlines after every 3.
            if (i + 1) % 3 == 0 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

impl GameBoard {
    pub fn new() -> Self {
        GameBoard([None; 3 * 3])
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
