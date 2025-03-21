mod game;

use std::{
    fs, io,
    os::unix::net::{SocketAddr, UnixDatagram},
};

use game::*;

const SERVER_SOCK_PATH: &str = "sk_xo.sock";
const PROMPT: &[u8] = b"> ";

fn prompt(game_board: &GameBoard, socket: &UnixDatagram, peer_addr: &SocketAddr) {
    let mut cursor = io::Cursor::new([0; 0x1000]);
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
        let mut game_board = GameBoard::new();

        prompt(&game_board, &socket, &first_peer_addr);
        prompt(&game_board, &socket, &second_peer_addr);

        loop {
            let (dgram_size, peer_addr) = socket.recv_from(&mut buf).unwrap();

            let pos: Result<u8, _> = std::str::from_utf8(&buf[..dgram_size - 1]).unwrap().parse();
            let pos = match pos {
                Ok(pos) => pos,
                Err(e) => {
                    socket.send_to_addr(e.to_string().as_bytes(), &peer_addr).unwrap();
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
                    let win_msg = format!("{} won!\n", player.name());
                    let _ = socket.send_to_addr(win_msg.as_bytes(), &first_peer_addr);
                    let _ = socket.send_to_addr(win_msg.as_bytes(), &second_peer_addr);
                    game_board = GameBoard::new();
                    prompt(&game_board, &socket, &first_peer_addr);
                    prompt(&game_board, &socket, &second_peer_addr);
                }
                Ok(false) => {
                    prompt(&game_board, &socket, &peer_addr);
                } Err(e) => {
                    socket.send_to_addr(e.to_string().as_bytes(), &peer_addr).unwrap();
                }
            }
        }
    }
}
