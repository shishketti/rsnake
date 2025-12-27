use piston_window::*;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::colors;
use crate::draw::*;
use crate::physics::{Direction, Position};
use crate::snake::Snake;

const FPS: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStatus {
    Playing,
    GameOver,
}

fn fps_as_duration(fps: f64) -> Duration {
    Duration::from_secs_f64(1.0 / fps)
}

fn calc_random_pos(width: u32, height: u32) -> Position {
    let mut rng = rand::thread_rng();

    Position {
        x: rng.gen_range(0..width as i32),
        y: rng.gen_range(0..height as i32),
    }
}

struct GameState {
    snake: Snake,
    fruit: Position,
    size: (u32, u32),
    score: u32,
    status: GameStatus,
    paused: bool,
    pending_direction: Option<Direction>,
    should_stop_thread: bool,
    apple_eaten_at: Option<Position>,
}

pub struct Game {
    state: Arc<Mutex<GameState>>,
    update_thread: Option<thread::JoinHandle<()>>,
}

impl Game {
    pub fn new(width: u32, height: u32) -> Self {
        let state = Arc::new(Mutex::new(GameState {
            snake: Snake::new(calc_random_pos(width, height)),
            fruit: calc_random_pos(width, height),
            size: (width, height),
            score: 0,
            status: GameStatus::Playing,
            paused: true,
            pending_direction: None,
            should_stop_thread: false,
            apple_eaten_at: None,
        }));

        Self {
            state,
            update_thread: None,
        }
    }

    pub fn start(&mut self) {
        {
            let mut state = self.state.lock().unwrap();
            state.paused = false;
            state.should_stop_thread = false;
        }

        // Start the game logic thread
        let state_clone = Arc::clone(&self.state);
        self.update_thread = Some(thread::spawn(move || {
            let tick_duration = fps_as_duration(FPS);
            let mut last_update = Instant::now();

            loop {
                let now = Instant::now();
                let elapsed = now.duration_since(last_update);

                if elapsed >= tick_duration {
                    last_update = now;

                    let mut state = state_clone.lock().unwrap();

                    if state.should_stop_thread {
                        break;
                    }

                    if state.status == GameStatus::GameOver {
                        // Keep the thread alive but don't update game logic
                        continue;
                    }

                    if state.paused {
                        continue;
                    }

                    // Apply pending direction change
                    if let Some(dir) = state.pending_direction.take() {
                        state.snake.set_dir(dir);
                    }

                    // Check for wall collision before updating
                    if state.snake.will_hit_wall(state.size.0, state.size.1) {
                        state.status = GameStatus::GameOver;
                        continue;
                    }

                    if !state.snake.is_tail_overlapping() && !state.snake.will_tail_overlapp() {
                        let snake_head_pos = state.snake.get_head_pos().clone();
                        let did_eat_fruit = snake_head_pos == state.fruit;

                        let (width, height) = state.size;
                        state.snake.update(width, height);

                        if did_eat_fruit {
                            state.snake.grow();
                            state.score = (state.snake.get_len() * 10) as u32;
                            state.apple_eaten_at = Some(state.fruit.clone());
                            state.fruit = calc_random_pos(width, height);
                        }
                    } else {
                        state.status = GameStatus::GameOver;
                    }
                }

                // Sleep briefly to avoid busy-waiting
                thread::sleep(Duration::from_millis(1));
            }
        }));
    }

    pub fn restart(&mut self) {
        // Stop the current thread
        {
            let mut state = self.state.lock().unwrap();
            state.should_stop_thread = true;
        }

        // Wait for the thread to finish
        if let Some(handle) = self.update_thread.take() {
            let _ = handle.join();
        }

        // Reset the game state
        {
            let mut state = self.state.lock().unwrap();
            let (width, height) = state.size;
            state.snake = Snake::new(calc_random_pos(width, height));
            state.fruit = calc_random_pos(width, height);
            state.score = 0;
            state.status = GameStatus::Playing;
            state.paused = false;
            state.pending_direction = None;
            state.should_stop_thread = false;
            state.apple_eaten_at = None;
        }

        // Start the game again
        self.start();
    }

    pub fn pause(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.paused = true;
    }

    pub fn get_status(&self) -> GameStatus {
        let state = self.state.lock().unwrap();
        state.status
    }

    pub fn draw(&self, ctx: Context, g: &mut G2d) {
        let state = self.state.lock().unwrap();
        draw_block(&ctx, g, colors::FRUIT, &state.fruit);
        state.snake.draw(&ctx, g);

        if state.status == GameStatus::GameOver {
            draw_overlay(&ctx, g, colors::OVERLAY, state.size)
        }
    }

    pub fn update(&mut self, _delta_time: f64) {
        // Game logic is now handled in a separate thread
        // This method is kept for API compatibility but does nothing
    }

    pub fn key_down(&mut self, key: keyboard::Key) {
        use keyboard::Key;

        // Check for restart key first (works even when game is over)
        if key == Key::R {
            let status = self.get_status();
            if status == GameStatus::GameOver {
                self.restart();
                return;
            }
        }

        let mut state = self.state.lock().unwrap();

        // Don't process movement keys if game is over
        if state.status == GameStatus::GameOver {
            return;
        }

        let dir = match key {
            Key::A | Key::Left => Some(Direction::Left),
            Key::W | Key::Up => Some(Direction::Up),
            Key::D | Key::Right => Some(Direction::Right),
            Key::S | Key::Down => Some(Direction::Down),
            _ => None,
        };

        if let Some(d) = dir {
            state.pending_direction = Some(d);
        }
    }

    pub fn get_score(&self) -> u32 {
        let state = self.state.lock().unwrap();
        state.score
    }

    pub fn take_apple_eaten(&mut self) -> Option<Position> {
        let mut state = self.state.lock().unwrap();
        state.apple_eaten_at.take()
    }
}
