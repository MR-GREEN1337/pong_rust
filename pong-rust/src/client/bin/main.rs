use std::net::UdpSocket;
use std::time::{Duration};
use common::{GameState, PaddleInput};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, terminal};
use std::io;
use std::process;

fn main() -> std::io::Result<()> {
    // UPDATED: Now requires three arguments: server, side, and name.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <server_address:port> <left|right> <your_name>", args[0]);
        process::exit(1);
    }
    let server_address = &args[1];
    let side = &args[2];
    let name = &args[3];

    println!("Attempting to connect to server at {} as player '{}' (name: {})", server_address, side, name);

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(server_address)?;
    socket.set_nonblocking(true)?;

    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::Clear(ClearType::All), cursor::Hide)?;

    let mut last_state: Option<GameState> = None;
    let mut buffer = [0; 1024];

    loop {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                let paddle_direction = match key.code {
                    KeyCode::Up => Some("Up".to_string()),
                    KeyCode::Down => Some("Down".to_string()),
                    KeyCode::Esc => break,
                    _ => None,
                };

                if let Some(direction) = paddle_direction {
                    // UPDATED: Include the name in the message.
                    let msg = PaddleInput {
                        side: side.clone(),
                        direction,
                        name: name.clone(),
                    };
                    let data = serde_json::to_vec(&msg).unwrap();
                    socket.send(&data)?;
                }
            }
        }

        loop {
            match socket.recv(&mut buffer) {
                Ok(amt) => {
                    if amt > 0 {
                        let state = serde_json::from_slice::<GameState>(&buffer[..amt]).unwrap();
                        last_state = Some(state);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to receive data: {}", e);
                    break;
                }
            }
        }
        
        if let Some(ref state) = last_state {
            state.draw(&side);
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    execute!(io::stdout(), cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}