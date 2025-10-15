use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;
use common::{GameState, PaddleInput};

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8080")?;
    socket.set_nonblocking(true)?;

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
                    if let Ok(mut input) = serde_json::from_slice::<PaddleInput>(&buffer[..amt]) {
                        let is_new_player = !clients.contains_key(&src);
                        
                        let assigned_side = if let Some(side) = clients.get(&src) {
                            side.clone()
                        } else {
                            if player_left_addr.is_none() {
                                println!("Player '{}' connected as 'left' from {}", input.name, src);
                                player_left_addr = Some(src);
                                clients.insert(src, "left".to_string());
                                "left".to_string()
                            } else if player_right_addr.is_none() {
                                println!("Player '{}' connected as 'right' from {}", input.name, src);
                                player_right_addr = Some(src);
                                clients.insert(src, "right".to_string());
                                "right".to_string()
                            } else {
                                println!("Ignoring connection from {} (server full)", src);
                                continue;
                            }
                        };
                        
                        // If it's a new player, set their name in the game state.
                        if is_new_player {
                            if assigned_side == "left" {
                                state.player_left_name = input.name.clone();
                            } else {
                                state.player_right_name = input.name.clone();
                            }
                        }

                        // Server is authoritative over the side.
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