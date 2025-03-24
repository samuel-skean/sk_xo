mod game;

use std::{
    fs,
    io::{self, Write},
    ops::{Deref, DerefMut},
    os::unix::net::{SocketAddr, UnixDatagram},
};

use game::Mark;

struct NetworkedGame {
    inner: game::Game,
    x_addr: SocketAddr,
    o_addr: SocketAddr,
}

impl NetworkedGame {
    fn player_addr(&self, player: Mark) -> &SocketAddr {
        match player {
            Mark::X => &self.x_addr,
            Mark::O => &self.o_addr,
        }
    }
}

impl Deref for NetworkedGame {
    type Target = game::Game;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for NetworkedGame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

const SERVER_SOCK_PATH: &str = "sk_xo.sock";
const PROMPT: &str = "> ";

// STRETCH: Man, it seems like writing to the cursor means there's more unwraps
// because each write can fail. They can, but only because we could run out of
// room, I think. How well optimized does this get? Does it check against other
// failure conditions too?

fn prompt(game: &NetworkedGame, socket: &UnixDatagram, peer_addr: &SocketAddr) {
    let mut cursor = io::Cursor::new([b'h'; 0x1000]);
    write!(cursor, "{}", game.inner).unwrap();
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
        let mut game = NetworkedGame {
            inner: game::Game::new(),
            x_addr: first_peer_addr,
            o_addr: second_peer_addr,
        };

        prompt(&game, &socket, &game.x_addr);
        socket
            .send_to_addr(b"The other player has joined.\n", &game.o_addr)
            .unwrap();

        loop {
            let (dgram_size, peer_addr) = socket.recv_from(&mut scratch).unwrap();

            let player = if peer_addr.as_pathname() == game.x_addr.as_pathname() {
                Mark::X
            } else if peer_addr.as_pathname() == game.o_addr.as_pathname() {
                Mark::O
            } else {
                eprintln!("Unexpected packet from {peer_addr:?}");
                continue;
            };
            // Should this be handled here or by an error in the game? I think
            // it should be handled here, because other errors cause a new
            // prompt. But I don't like how this interrupts the integer parsing
            // code.
            if game.current_player() != player {
                socket.send_to_addr(b"It's not your turn.\n", &peer_addr).unwrap();
                continue;
            }

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

            match game.make_move(pos) {
                Ok(true) => {
                    // TODO: Always print this on *a* newline.
                    write!(scratch_cursor, "{player} won!\n").unwrap();
                    let written_to_scratch =
                        &scratch_cursor.get_ref()[..scratch_cursor.position() as usize];
                    let _ = socket.send_to_addr(written_to_scratch, &game.x_addr);
                    let _ = socket.send_to_addr(written_to_scratch, &game.o_addr);
                    game.inner = game::Game::new();
                    prompt(&game, &socket, &game.x_addr);
                }
                Ok(false) => {
                    prompt(&game, &socket, game.player_addr(player.opponent()));
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
