use std::{fs, io, os::unix::net::UnixDatagram};

const SERVER_SOCK_PATH: &str = "skean_tic_tac_toe.sock";

#[derive(Debug, Clone, Copy)]
enum GameSquare { X, O }

struct GameBoard([Option<GameSquare>; 3 * 3]);

impl GameBoard {
    fn encode_to(&self, mut buf: &mut [u8]) -> usize {
        assert!(buf.len() > (3 * 3) + 3); // Big enough for the board and its newlines.
        for (i, square) in self.0.iter().enumerate() {
            buf[0] = match square {
                Some(GameSquare::X) => b'X',
                Some(GameSquare::O) => b'O',
                None => (i + 1) as u8 + b'0',
            };
            // Advance one character.
            buf = &mut buf[1..];
            // Put newlines after every 3.
            if (i + 1) % 3 == 0 {
                buf[0] = b'\n';
                buf = &mut buf[1..];
            }
        }
        (3 * 3) + 3
    }
}

fn main() {
    match fs::remove_file(SERVER_SOCK_PATH) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => { panic!("Could not ensure {SERVER_SOCK_PATH} was removed, error: {e}"); } 
    }
    let listener =
        UnixDatagram::bind(SERVER_SOCK_PATH).expect("Unable to create listener socket.");

    loop {
        let mut buf = [0; 0x1000];
        let (dgram_size, first_peer_addr) = listener.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..dgram_size], b"\n");

        dbg!(&first_peer_addr);

        listener.connect_addr(&first_peer_addr).unwrap();
        loop {
            let game_board = GameBoard([None; 3 * 3]);
            let bytes_encoded = game_board.encode_to(&mut buf);
            listener.send(&buf[..bytes_encoded]).unwrap();
            let (dgram_size, peer_addr) = listener.recv_from(&mut buf).unwrap();
            assert_eq!(first_peer_addr.as_pathname().unwrap(), peer_addr.as_pathname().unwrap());
            
            

        }

    }
}
