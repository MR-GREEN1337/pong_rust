use core::panic;
use std::{io::{stdout, Write}, time::{Duration, Instant}};

use serde::{Serialize, Deserialize};
use crossterm::{cursor, execute, terminal::{self, ClearType}};

#[derive(Serialize, Deserialize)]
pub struct PaddleInput {
    pub side: String,
    pub direction: String,
    pub name: String, // NEW: Player name for display
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
    // NEW: Fields for scoring and player names
    pub score_left: u32,
    pub score_right: u32,
    pub player_left_name: String,
    pub player_right_name: String,
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
            // Initialize new fields
            score_left: 0,
            score_right: 0,
            player_left_name: "Waiting...".to_string(),
            player_right_name: "Waiting...".to_string(),
        }
    }

    // Resets the ball to the center after a score
    fn reset_ball(&mut self) {
        self.ball_x = self.screen_width as f32 / 2.0;
        self.ball_y = self.screen_height as f32 / 2.0;
        // Reverse direction to serve to the other player
        self.vel_x = -self.vel_x;
    }

    pub fn update(&mut self, input: Option<PaddleInput>, last_update: &mut Instant) {
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
            
            // Constrain paddle movement within the borders (1 top, height-2 bottom)
            if *new_pos < 3.0 {
                *new_pos = 3.0;
            } else if *new_pos > self.screen_height as f32 - 4.0 {
                *new_pos = self.screen_height as f32 - 4.0;
            }
        }

        if last_update.elapsed() > self.tick {
            self.ball_x += self.vel_x;
            self.ball_y += self.vel_y;

            // Bounce on top/bottom walls (inside the border)
            if self.ball_y <= 1.0 || self.ball_y >= (self.screen_height as f32) - 2.0 {
                self.vel_y = -self.vel_y;
            }

            // Bounce on paddle left (inside the border)
            if self.ball_x <= 3.0 && (self.ball_y - self.paddle_left).abs() < 3.0 {
                self.vel_x = self.vel_x.abs();
            }

            // Bounce on paddle right (inside the border)
            if self.ball_x >= (self.screen_width as f32) - 3.0 && (self.ball_y - self.paddle_right).abs() < 3.0 {
                self.vel_x = -self.vel_x.abs();
            }

            // Scoring logic
            if self.ball_x < 1.0 { // Left side out of bounds
                self.score_right += 1;
                self.reset_ball();
            } else if self.ball_x > (self.screen_width as f32) - 1.0 { // Right side out of bounds
                self.score_left += 1;
                self.reset_ball();
            }

            *last_update = Instant::now();
        }
    }

    // REVAMPED: Draws the game state with borders, scores, and names
    pub fn draw(&self, side: &str) {
        let mut out = stdout();
        let width = self.screen_width;
        let height = self.screen_height;

        execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::FromCursorDown)).unwrap();

        // --- Draw Header ---
        let score_line = format!(
            "{} [{}]  -  [{}] {}",
            self.player_left_name, self.score_left, self.score_right, self.player_right_name
        );
        // Center the score line
        writeln!(out, "{:^width$}\r", score_line, width = width).unwrap();
        
        // --- Build Screen Buffer ---
        let mut screen = vec![vec![' '; width]; height];

        // Draw borders and center line
        for y in 1..height -1 {
            screen[y][0] = '║';
            screen[y][width - 1] = '║';
            if y % 2 == 0 { // Dashed center line
               screen[y][width / 2] = '┊';
            }
        }
        for x in 1..width -1 {
            screen[1][x] = '═';
            screen[height - 1][x] = '═';
        }
        screen[1][0] = '╔';
        screen[1][width - 1] = '╗';
        screen[height - 1][0] = '╚';
        screen[height - 1][width - 1] = '╝';
        screen[1][width / 2] = '╦';
        screen[height-1][width / 2] = '╩';


        // Draw paddles
        let half_paddle = 2;
        for dy in -half_paddle..=half_paddle {
            let yl = (self.paddle_left as i32 + dy).clamp(0, height as i32 - 1);
            let yr = (self.paddle_right as i32 + dy).clamp(0, height as i32 - 1);
            screen[yl as usize][2] = '█'; // Left paddle at x=2
            screen[yr as usize][width - 3] = '█'; // Right paddle at x=width-3
        }

        // Draw ball
        let bx = self.ball_x.clamp(0.0, (width - 1) as f32) as usize;
        let by = self.ball_y.clamp(0.0, (height - 1) as f32) as usize;
        screen[by][bx] = '●';

        // --- Print Screen Buffer ---
        // Skip the top row (already used for score) and write buffer line by line
        for y in 1..height {
            writeln!(out, "{}\r", screen[y].iter().collect::<String>()).unwrap();
        }

        writeln!(out, "\r\nYou are player '{}'. Use Up/Down arrows. Press ESC to quit.", side).unwrap();
        out.flush().unwrap();
    }
}