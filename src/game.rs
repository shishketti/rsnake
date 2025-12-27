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
// const RESTART_TIME: f64 = 1.0;

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
    over: bool,
    paused: bool,
    pending_direction: Option<Direction>,
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
            over: false,
            paused: true,
            pending_direction: None,
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

                    if state.over {
                        break;
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
                        state.over = true;
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
                            state.fruit = calc_random_pos(width, height);
                        }
                    } else {
                        state.over = true;
                    }
                }

                // Sleep briefly to avoid busy-waiting
                thread::sleep(Duration::from_millis(1));
            }
        }));
    }

    pub fn pause(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.paused = true;
    }

    // pub fn toggle_game_state(&mut self) {
    //     if self.paused {
    //         self.start();
    //     } else {
    //         self.pause();
    //     }
    // }

    pub fn draw(&self, ctx: Context, g: &mut G2d) {
        let state = self.state.lock().unwrap();
        draw_block(&ctx, g, colors::FRUIT, &state.fruit);
        state.snake.draw(&ctx, g);

        if state.over {
            draw_overlay(&ctx, g, colors::OVERLAY, state.size)
        }
    }

    pub fn update(&mut self, _delta_time: f64) {
        // Game logic is now handled in a separate thread
        // This method is kept for API compatibility but does nothing
    }

    pub fn key_down(&mut self, key: keyboard::Key) {
        use keyboard::Key;

        let mut state = self.state.lock().unwrap();

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

    // IMPORTANT!! -

    // fn update_snake(&mut self, dir: Option<Direction>) {
    //     if self.check_if_snake_alive(dir) {
    //         self.snake.move_forward(dir);
    //         self.check_eating();
    //     } else {
    //         self.game_over = true;
    //     }
    //     self.waiting_time = 0.0;
    // }
}

// fn calc_not_overlapping_pos(pos_vec: Vec<Position>, width: u32, height: u32) {
//     let mut fruit_pos: Position = calc_random_pos(width, height);

//     loop {
//         // if snake_pos.y != fruit_pos.y {
//         //     break;
//         // }

//         for pos in pos_vec {
//             if
//         }

//         snake_pos = calc_random_pos(width, height);
//         fruit_pos = calc_random_pos(width, height);
//     }
// }
