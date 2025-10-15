use core::panic;
use std::{io::{stdout, Write}, time::{Duration, Instant}};

use serde::{Serialize, Deserialize};
use crossterm::{cursor, execute, terminal::{self, ClearType}};

#[derive(Serialize, Deserialize)]
pub struct PaddleInput {
    pub side: String,
    pub direction: String,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub ball_x: f32,
    pub ball_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub paddle_left: f32,
    pub paddle_right: f32,
    pub screen_height: usize,
    pub screen_width: usize,
    pub tick: Duration,    
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            ball_x: 40.0,
            ball_y: 12.0,
            vel_x: 1.0,
            vel_y: 0.5,
            paddle_left: 10.0,
            paddle_right: 10.0,
            screen_height: 25,
            screen_width: 80,
            tick: Duration::from_millis(50),
        }
    }

    // Update game state with optional paddle input and last update time.
    pub fn update(&mut self, input: Option<PaddleInput>, last_update: &mut Instant) {
        // Firstly check if some paddle input have been received
        if let Some(paddle) = input {
            let new_pos = match paddle.side.as_str() {
                "left" => &mut self.paddle_left,
                "right" => &mut self.paddle_right,
                _ => { panic!("Wrong paddle given") }
            };

            match paddle.direction.as_str() {
                "Up" => *new_pos -= 1.0,
                "Down" => *new_pos += 1.0,
                _ => panic!("Wrong direction given")
            };
            
            if *new_pos < 2.0 {
                *new_pos = 2.0;
            } else if *new_pos > self.screen_height as f32 - 3.0 {
                *new_pos = self.screen_height as f32 - 3.0;
            }
        }

        // Prevent refreshing too much, otherwise the ball will be too fast.
        // This can be tweaked with `self.tick`.
        if last_update.elapsed() > self.tick {
            // Update ball coordinates
            self.ball_x += self.vel_x;
            self.ball_y += self.vel_y;

            // Bounce on top/bottom
            if self.ball_y <= 0.0 || self.ball_y >= (self.screen_height as f32) - 1.0 {
                self.vel_y = -self.vel_y;
            }

            // Bounce on paddle left
            if self.ball_x <= 2.0 && (self.ball_y - self.paddle_left).abs() < 3.0 {
                self.vel_x = self.vel_x.abs();
            }

            // Bounce on paddle right
            if self.ball_x >= (self.screen_width as f32) - 2.0 && (self.ball_y - self.paddle_right).abs() < 3.0 {
                self.vel_x = -self.vel_x.abs();
            }

            *last_update = Instant::now();
        }
    }

    // Draws the game state
    pub fn draw(&self, side: &str) {
        let mut out = stdout();
        let width = self.screen_width;
        let height = self.screen_height;

        // Always reset cursor position to (0, 0)
        execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::FromCursorDown)).unwrap();

        // Build screen buffer
        let mut screen = vec![vec![' '; width]; height];

        let half_paddle = 2;
        for dy in -half_paddle..=half_paddle {
            let yl = (self.paddle_left as i32 + dy).clamp(0, height as i32 - 1);
            let yr = (self.paddle_right as i32 + dy).clamp(0, height as i32 - 1);
            screen[yl as usize][1] = '|';
            screen[yr as usize][width - 2] = '|';
        }

        let bx = self.ball_x.clamp(0.0, (width - 1) as f32) as usize;
        let by = self.ball_y.clamp(0.0, (height - 1) as f32) as usize;
        screen[by][bx] = '●';

        // Manual write with \r\n (important!)
        for row in screen {
            writeln!(out, "{}\r", row.into_iter().collect::<String>()).unwrap();
        }

        writeln!(out, "\r\nYou are player '{}'. Press arrow Up/Down to move, ESC to quit.", side).unwrap();
        out.flush().unwrap();
    }
}
