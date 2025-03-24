use std::fmt;

use thiserror::Error;

const SIDE_LENGTH: u8 = 3;
const BOARD_SIZE: u8 = SIDE_LENGTH * SIDE_LENGTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    X,
    O,
}

impl Mark {
    pub fn opponent(self) -> Self {
        match self {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        }
    }
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Mark::X => "X",
            Mark::O => "O",
        })
    }
}

pub struct Game {
    board: [Option<Mark>; BOARD_SIZE as usize],
    current_player: Mark,
}

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("Index was too big. Pick a number from 1 through {BOARD_SIZE}.")]
    PosTooBig(u8),
    #[error("{0} had already moved there.")]
    AlreadyMoved(Mark),
}

// TODO: Should GameBoard be a separate type? This doesn't exactly print the whole game...
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, square) in self.board.iter().enumerate() {
            if let Some(filled_square) = square {
                write!(f, "{filled_square}")?;
            } else {
                write!(f, "{}", i + 1)?;
            }
            // Put newlines after every SIDE_LENGTH.
            if (i + 1) % SIDE_LENGTH as usize == 0 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: [None; (SIDE_LENGTH * SIDE_LENGTH) as usize],
            current_player: Mark::X,
        }
    }

    pub fn current_player(&self) -> Mark {
        self.current_player
    }

    /// `pos` is 1-indexed.
    pub fn make_move(&mut self, pos: u8) -> Result<bool, MoveError> {
        let player = self.current_player;
        let pos = pos - 1;
        if pos >= BOARD_SIZE {
            Err(MoveError::PosTooBig(pos))
        } else if let Some(p) = self.board[pos as usize] {
            Err(MoveError::AlreadyMoved(p))
        } else {
            self.board[pos as usize] = Some(player);
            let winner = self.winner();
            if winner != None && winner != Some(player) {
                panic!("Made a move and the other guy won.");
            }
            self.current_player = player.opponent();
            Ok(winner == Some(player))
        }
    }

    fn winner(&self) -> Option<Mark> {
        // STRETCH: A "cleaner" implementation might involve chunking an
        // iterator of all the squares we have to check. The standard library
        // doesn't let you chunk arbitrary iterators, though - just slices. So,
        // we could copy the elements to a slice (but that's tricky to do
        // without allocation, which I want to avoid). Or, we could use
        // something like `itertools`'s [`chunk`
        // method](https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.chunks),
        // which by the nature of its task must allocate sometimes (but not
        // here, where we'd consume the chunks entirely in order). Of course, we
        // could also write a loop ourselves to handle the chunks.
        fn check_series(series: impl IntoIterator<Item = Option<Mark>>) -> Option<Mark> {
            let mut series = series.into_iter();
            let first = series.next().unwrap();
            if series.all(|s| s == first) {
                first
            } else {
                None
            }
        }
        
        fn check_pattern(
            pattern: impl IntoIterator<Item = impl IntoIterator<Item = Option<Mark>>>,
        ) -> Option<Mark> {
            for series in pattern {
                if let Some(winner) = check_series(series) {
                    return Some(winner);
                }
            }
            None
        }

        let rows = self
            .board
            .chunks_exact(SIDE_LENGTH as usize)
            .map(|s| s.iter().copied());
        let columns = (0..SIDE_LENGTH)
            .map(|y| (0..SIDE_LENGTH).map(move |x| self.board[(3 * x + y) as usize]));
        let downwards_diagonal =
            (0..SIDE_LENGTH).map(|i| self.board[(i + SIDE_LENGTH * i) as usize]);
        let upwards_diagonal =
            (0..SIDE_LENGTH).map(|i| self.board[(SIDE_LENGTH - 1 - i + SIDE_LENGTH * i) as usize]);

        check_pattern(rows)
            .or_else(|| check_pattern(columns))
            .or_else(|| check_series(downwards_diagonal))
            .or_else(|| check_series(upwards_diagonal))
    }
}
