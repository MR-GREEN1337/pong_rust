use std::net::UdpSocket;
use std::time::{Duration};
use common::{GameState, PaddleInput};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, terminal};
use std::io;
use std::process;

fn main() -> std::io::Result<()> {
    // We now expect two command-line arguments: the server address and the player side.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <server_address:port> <left|right>", args[0]);
        process::exit(1);
    }
    let server_address = &args[1];
    let side = &args[2];

    println!("Attempting to connect to server at {} as player '{}'...", server_address, side);

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    // Connect to the server address provided via the command line.
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
                    let msg = PaddleInput {
                        side: side.clone(),
                        direction,
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