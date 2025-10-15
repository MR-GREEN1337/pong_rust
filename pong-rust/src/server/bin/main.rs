use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;
use common::{GameState, PaddleInput};

fn main() -> std::io::Result<()> {
    // Change this line to bind to "0.0.0.0"
    // This allows the server to accept connections from other computers on the network.
    let socket = UdpSocket::bind("0.0.0.0:8080")?;
    socket.set_nonblocking(true)?;

    // Use the local IP for the print message so the user knows what to share.
    let server_addr = socket.local_addr()?;
    println!("Server listening on {} (share this address with other players)", server_addr);

    let mut clients: HashMap<SocketAddr, String> = HashMap::new();
    let mut player_left_addr: Option<SocketAddr> = None;
    let mut player_right_addr: Option<SocketAddr> = None;

    let mut state = GameState::new();
    let mut last_update = Instant::now();
    let mut buffer = [0; 1024];

    loop {
        loop {
            match socket.recv_from(&mut buffer) {
                Ok((amt, src)) => {
                    let assigned_side = if let Some(side) = clients.get(&src) {
                        side.clone()
                    } else {
                        if player_left_addr.is_none() {
                            println!("Player 'left' connected from {}", src);
                            player_left_addr = Some(src);
                            clients.insert(src, "left".to_string());
                            "left".to_string()
                        } else if player_right_addr.is_none() {
                            println!("Player 'right' connected from {}", src);
                            player_right_addr = Some(src);
                            clients.insert(src, "right".to_string());
                            "right".to_string()
                        } else {
                            println!("Ignoring connection from {} (server full)", src);
                            continue;
                        }
                    };

                    if let Ok(mut input) = serde_json::from_slice::<PaddleInput>(&buffer[..amt]) {
                        input.side = assigned_side;
                        state.update(Some(input), &mut last_update);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    eprintln!("Could not receive data: {}", e);
                    break;
                }
            }
        }

        state.update(None, &mut last_update);

        if !clients.is_empty() {
            let data = serde_json::to_vec(&state).unwrap();
            for addr in clients.keys() {
                socket.send_to(&data, addr)?;
            }
        }

        std::thread::sleep(state.tick / 2);
    }
}