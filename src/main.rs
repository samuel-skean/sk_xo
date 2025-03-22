mod game;

use std::{
    fs,
    io::{self, Write},
    os::unix::net::{SocketAddr, UnixDatagram},
};

use game::*;

const SERVER_SOCK_PATH: &str = "sk_xo.sock";
const PROMPT: &str = "> ";

// TODO: Man, it seems like writing to the cursor means there's more unwraps
// because each write can fail. They can, but only because we could run out of
// room, I think. How well optimized does this get? Does it check against other
// failure conditions too?

fn prompt(game_board: &GameBoard, socket: &UnixDatagram, peer_addr: &SocketAddr) {
    let mut cursor = io::Cursor::new([b'h'; 0x1000]);
    write!(cursor, "{game_board}").unwrap();
    socket
        .send_to_addr(&cursor.get_ref()[..cursor.position() as usize], peer_addr)
        .unwrap();

    socket.send_to_addr(PROMPT.as_bytes(), peer_addr).unwrap();
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
        let mut scratch = [b'h'; 0x1000];
        // TODO: Any way to just discard these initial packets?
        let (_, first_peer_addr) = socket.recv_from(&mut scratch).unwrap();

        let mut second_peer_addr;
        // This sure is code style:
        while {
            (_, second_peer_addr) = socket.recv_from(&mut scratch).unwrap();
            second_peer_addr.as_pathname() == first_peer_addr.as_pathname()
        } {}

        dbg!(&first_peer_addr);
        let mut game_board = GameBoard::new();

        prompt(&game_board, &socket, &first_peer_addr);
        prompt(&game_board, &socket, &second_peer_addr);

        loop {
            let (dgram_size, peer_addr) = socket.recv_from(&mut scratch).unwrap();

            let pos: Result<u8, _> = std::str::from_utf8(&scratch[..dgram_size - 1])
                .unwrap()
                .parse();

            let mut scratch_cursor = io::Cursor::new(&mut scratch[..]);

            let pos = match pos {
                Ok(pos) => pos,
                Err(e) => {
                    write!(scratch_cursor, "{e}\n{PROMPT}").unwrap();
                    socket
                        .send_to_addr(
                            &scratch_cursor.get_ref()[..scratch_cursor.position() as usize],
                            &peer_addr,
                        )
                        .unwrap();
                    continue;
                }
            };

            let player = if peer_addr.as_pathname() == first_peer_addr.as_pathname() {
                GameSquare::X
            } else if peer_addr.as_pathname() == second_peer_addr.as_pathname() {
                GameSquare::O
            } else {
                eprintln!("Unexpected packet from {peer_addr:?}");
                continue;
            };
            match game_board.make_move(player, pos) {
                Ok(true) => {
                    // STRETCH: Might be nice to tell the player if *they* won.
                    // TODO: Always print this on *a* newline.
                    write!(scratch_cursor, "{player} won!\n").unwrap();
                    let written_to_scratch =
                        &scratch_cursor.get_ref()[..scratch_cursor.position() as usize];
                    let _ = socket.send_to_addr(written_to_scratch, &first_peer_addr);
                    let _ = socket.send_to_addr(written_to_scratch, &second_peer_addr);
                    game_board = GameBoard::new();
                    prompt(&game_board, &socket, &first_peer_addr);
                    prompt(&game_board, &socket, &second_peer_addr);
                }
                Ok(false) => {
                    prompt(&game_board, &socket, &peer_addr);
                }
                Err(e) => {
                    write!(scratch_cursor, "{e}\n{PROMPT}").unwrap();
                    socket
                        .send_to_addr(
                            &scratch_cursor.get_ref()[..scratch_cursor.position() as usize],
                            &peer_addr,
                        )
                        .unwrap();
                }
            }
        }
    }
}
