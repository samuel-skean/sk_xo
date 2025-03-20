use std::{
    fs, io,
    os::unix::net::{SocketAddr, UnixDatagram},
};

const SERVER_SOCK_PATH: &str = "skean_tic_tac_toe.sock";
const PROMPT: &[u8] = b"> ";

#[derive(Debug, Clone, Copy)]
enum GameSquare {
    X,
    O,
}

struct GameBoard([Option<GameSquare>; 3 * 3]);

impl GameBoard {
    fn new() -> Self {
        GameBoard([None; 3 * 3])
    }

    fn encode_to(&self, sink: &mut impl io::Write) {
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

fn prompt(game_board: &GameBoard, socket: &UnixDatagram, peer_addr: &SocketAddr, buf: &mut [u8]) {
    let mut cursor = io::Cursor::new(buf);
    game_board.encode_to(&mut cursor);
    socket
        .send_to_addr(&cursor.get_ref()[..cursor.position() as usize], peer_addr)
        .unwrap();

    socket.send_to_addr(&PROMPT, peer_addr).unwrap();
}

fn main() {
    match fs::remove_file(SERVER_SOCK_PATH) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => {
            panic!("Could not ensure {SERVER_SOCK_PATH} was removed, error: {e}");
        }
    }
    let socket = UnixDatagram::bind(SERVER_SOCK_PATH).expect("Unable to create listener socket.");

    loop {
        let mut buf = [b'h'; 0x1000];
        // TODO: Any way to just discard these initial packets?
        let (_, first_peer_addr) = socket.recv_from(&mut buf).unwrap();

        let mut second_peer_addr;
        // This sure is code style:
        while {
            (_, second_peer_addr) = socket.recv_from(&mut buf).unwrap();
            second_peer_addr.as_pathname() == first_peer_addr.as_pathname()
        } {}

        dbg!(&first_peer_addr);
        let game_board = GameBoard::new();

        prompt(&game_board, &socket, &first_peer_addr, &mut buf);
        prompt(&game_board, &socket, &second_peer_addr, &mut buf);

        loop {
            let (_, peer_addr) = socket.recv_from(&mut buf).unwrap();

            if peer_addr.as_pathname() == first_peer_addr.as_pathname() {
                prompt(&game_board, &socket, &first_peer_addr, &mut buf);
            } else if peer_addr.as_pathname() == second_peer_addr.as_pathname() {
                prompt(&game_board, &socket, &second_peer_addr, &mut buf);
            } else {
                eprintln!("Unexpected packet from {peer_addr:?}");
            }
        }
    }
}
