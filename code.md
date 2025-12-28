### src/colors.rs

```rust
use piston_window::types::Color;

pub const BACKGROUND: Color = [0.0, 0.0, 0.0, 1.0];
pub const SCORE: Color = [1.0, 1.0, 1.0, 1.0];
pub const SNAKE: Color = [0.1, 0.9, 0.1, 1.0];
pub const FRUIT: Color = [1.0, 0.0, 0.0, 1.0];
pub const OVERLAY: Color = [1.0, 0.0, 0.0, 0.5];

// Grid background colors
pub const GRID_LIGHT: Color = [0.2, 0.35, 0.15, 1.0]; // Subtle green
pub const GRID_DARK: Color = [0.25, 0.2, 0.15, 1.0]; // Subtle brown
```

---

### src/draw.rs

```rust
use crate::colors;
use crate::physics::{Direction, Position};
use noise::{NoiseFn, Perlin};
use piston_window::types::Color;
use piston_window::{rectangle, Context, G2d};
use rand::Rng;

pub const BLOCK_SIZE: f64 = 25.0;

pub struct Background {
    colors: Vec<Vec<Color>>,
}

impl Background {
    pub fn new(width: u32, height: u32) -> Self {
        let mut rng = rand::thread_rng();
        let seed: u32 = rng.gen();
        let perlin = Perlin::new(seed);

        let scale = 0.15; // Controls how "zoomed in" the noise is

        let mut colors = Vec::with_capacity(width as usize);
        for x in 0..width {
            let mut row = Vec::with_capacity(height as usize);
            for y in 0..height {
                let noise_val = perlin.get([x as f64 * scale, y as f64 * scale]);
                // noise_val is roughly -1.0 to 1.0, normalize to 0.0 to 1.0
                let t = (noise_val + 1.0) / 2.0;

                // Interpolate between green and brown
                let color = [
                    colors::GRID_LIGHT[0]
                        + (colors::GRID_DARK[0] - colors::GRID_LIGHT[0]) * t as f32,
                    colors::GRID_LIGHT[1]
                        + (colors::GRID_DARK[1] - colors::GRID_LIGHT[1]) * t as f32,
                    colors::GRID_LIGHT[2]
                        + (colors::GRID_DARK[2] - colors::GRID_LIGHT[2]) * t as f32,
                    1.0,
                ];
                row.push(color);
            }
            colors.push(row);
        }

        Background { colors }
    }

    pub fn draw(&self, ctx: &Context, g: &mut G2d) {
        for (x, row) in self.colors.iter().enumerate() {
            for (y, color) in row.iter().enumerate() {
                rectangle(
                    *color,
                    [
                        x as f64 * BLOCK_SIZE,
                        y as f64 * BLOCK_SIZE,
                        BLOCK_SIZE,
                        BLOCK_SIZE,
                    ],
                    ctx.transform,
                    g,
                );
            }
        }
    }
}

pub fn draw_block(ctx: &Context, g: &mut G2d, c: Color, pos: &Position) {
    rectangle(
        c,
        [
            pos.x as f64 * BLOCK_SIZE,
            pos.y as f64 * BLOCK_SIZE,
            BLOCK_SIZE,
            BLOCK_SIZE,
        ],
        ctx.transform,
        g,
    );
}

pub fn draw_snake_head(ctx: &Context, g: &mut G2d, c: Color, pos: &Position, dir: &Direction) {
    draw_block(ctx, g, c, pos);

    fn draw_eye(ctx: &Context, g: &mut G2d, x: f64, y: f64) {
        rectangle(colors::BACKGROUND, [x, y, 5.0, 5.0], ctx.transform, g);
    }

    let (x, y) = (
        blocks_in_pixels(pos.x as u32) as f64,
        blocks_in_pixels(pos.y as u32) as f64,
    );

    let block = blocks_in_pixels(1) as f64;

    match dir {
        Direction::Up => {
            draw_eye(ctx, g, x + 5.0, y + 5.0);
            draw_eye(ctx, g, x + block - 10.0, y + 5.0);
        }
        Direction::Right => {
            draw_eye(ctx, g, x + block - 10.0, y + 5.0);
            draw_eye(ctx, g, x + block - 10.0, y + block - 10.0);
        }
        Direction::Down => {
            draw_eye(ctx, g, x + 5.0, y + block - 10.0);
            draw_eye(ctx, g, x + block - 10.0, y + block - 10.0);
        }
        Direction::Left => {
            draw_eye(ctx, g, x + 5.0, y + 5.0);
            draw_eye(ctx, g, x + 5.0, y + block - 10.0);
        }
    }
}

pub fn draw_fruit(_ctx: &Context, _g: &mut G2d, _c: Color, _pos: &Position) {}

pub fn draw_overlay(ctx: &Context, g: &mut G2d, c: Color, size: (u32, u32)) {
    rectangle(
        c,
        [
            0.0,
            0.0,
            blocks_in_pixels(size.0) as f64,
            blocks_in_pixels(size.1) as f64,
        ],
        ctx.transform,
        g,
    );
}

pub fn blocks_in_pixels(n: u32) -> u32 {
    n * BLOCK_SIZE as u32
}
```

---

### src/game.rs

```rust
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
```

---

### src/main.rs

```rust
/* Copyright (C) 2019 by Mara Schulke */

/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.
This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

mod colors;
mod draw;
mod game;
mod particles;
mod physics;
mod snake;

use draw::{blocks_in_pixels, Background};
use game::{Game, GameStatus};
use particles::ParticleSystem;
use piston_window::*;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;

const WINDOW_TITLE: &'static str = "rsnake";
const WIDTH: u32 = 25;
const HEIGHT: u32 = 25;

fn main() {
    let size = [blocks_in_pixels(WIDTH), blocks_in_pixels(HEIGHT)];

    let mut window: PistonWindow = WindowSettings::new(WINDOW_TITLE, size)
        .resizable(false)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let ref font = assets.join("retro-gaming.ttf");
    let _factory = window.factory.clone();
    let mut glyphs = Glyphs::new(
        font,
        TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into(),
        },
        TextureSettings::new(),
    )
    .unwrap();

    // Set up audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Background music (looping)
    let music_sink = Sink::try_new(&stream_handle).unwrap();
    let music_file = BufReader::new(File::open(assets.join("snakejazz.ogg")).unwrap());
    let music_source = Decoder::new(music_file).unwrap().repeat_infinite();
    music_sink.append(music_source);
    music_sink.set_volume(0.5);
    let mut music_muted = false;

    // Sound effect sink for apple
    let sfx_sink = Sink::try_new(&stream_handle).unwrap();
    sfx_sink.set_volume(0.8);

    let mut main: Game = Game::new(WIDTH, HEIGHT);
    let background = Background::new(WIDTH, HEIGHT);
    let mut particle_system = ParticleSystem::new();
    main.start();

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            // Toggle music mute with M key
            if key == Key::M {
                music_muted = !music_muted;
                if music_muted {
                    music_sink.set_volume(0.0);
                } else {
                    music_sink.set_volume(0.5);
                }
            }
            main.key_down(key);
        }

        // Check if apple was eaten
        if let Some(pos) = main.take_apple_eaten() {
            // Play sound effect
            if let Ok(file) = File::open(assets.join("appleobtained.ogg")) {
                let reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(reader) {
                    sfx_sink.append(source);
                }
            }

            // Spawn particles at apple position
            particle_system.spawn_at(pos.x, pos.y, colors::FRUIT);
        }

        window.draw_2d(&event, |ctx, g, device| {
            clear(colors::BACKGROUND, g);
            background.draw(&ctx, g);

            // Draw the score
            let score_str = main.get_score().to_string();
            text::Text::new_color(colors::SCORE, 20)
                .draw(
                    score_str.as_ref(),
                    &mut glyphs,
                    &ctx.draw_state,
                    ctx.transform.trans(0.0, 20.0),
                    g,
                )
                .unwrap();

            // Draw mute button in top right
            let window_width = blocks_in_pixels(WIDTH) as f64;
            let mute_text = if music_muted { "[M] OFF" } else { "[M] ON" };
            text::Text::new_color(colors::SCORE, 20)
                .draw(
                    mute_text,
                    &mut glyphs,
                    &ctx.draw_state,
                    ctx.transform.trans(window_width - 100.0, 20.0),
                    g,
                )
                .unwrap();

            // Draw the game elements (snake, fruit, etc.)
            main.draw(ctx, g);

            // Draw particles
            particle_system.draw(&ctx, g);

            // Draw game over text if the game is over
            if main.get_status() == GameStatus::GameOver {
                let window_width = blocks_in_pixels(WIDTH) as f64;
                let window_height = blocks_in_pixels(HEIGHT) as f64;

                // Draw "GAME OVER" text
                text::Text::new_color(colors::SCORE, 32)
                    .draw(
                        "GAME OVER",
                        &mut glyphs,
                        &ctx.draw_state,
                        ctx.transform
                            .trans(window_width / 2.0 - 90.0, window_height / 2.0 - 20.0),
                        g,
                    )
                    .unwrap();

                // Draw "Press R to Restart" text
                text::Text::new_color(colors::SCORE, 16)
                    .draw(
                        "Press R to Restart",
                        &mut glyphs,
                        &ctx.draw_state,
                        ctx.transform
                            .trans(window_width / 2.0 - 80.0, window_height / 2.0 + 20.0),
                        g,
                    )
                    .unwrap();
            }

            // Update glyphs texture context after drawing
            glyphs.factory.encoder.flush(device);
        });

        event.update(|arg| {
            main.update(arg.dt);
            particle_system.update(arg.dt);
        });
    }
}
```

---

### src/particles.rs

```rust
use crate::draw::BLOCK_SIZE;
use piston_window::types::Color;
use piston_window::{rectangle, Context, G2d};
use rand::Rng;

const PARTICLE_COUNT: usize = 12;
const PARTICLE_LIFETIME: f64 = 0.8;
const PARTICLE_SPEED: f64 = 150.0;
const PARTICLE_SIZE: f64 = 6.0;

pub struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    lifetime: f64,
    color: Color,
}

impl Particle {
    pub fn new(x: f64, y: f64, color: Color) -> Self {
        let mut rng = rand::thread_rng();
        let angle: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
        let speed: f64 = rng.gen_range(PARTICLE_SPEED * 0.5..PARTICLE_SPEED);

        Particle {
            x,
            y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            lifetime: PARTICLE_LIFETIME,
            color,
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.lifetime -= dt;

        // Slow down over time
        self.vx *= 0.98;
        self.vy *= 0.98;
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }

    pub fn draw(&self, ctx: &Context, g: &mut G2d) {
        let alpha = (self.lifetime / PARTICLE_LIFETIME) as f32;
        let color = [self.color[0], self.color[1], self.color[2], alpha];

        rectangle(
            color,
            [
                self.x - PARTICLE_SIZE / 2.0,
                self.y - PARTICLE_SIZE / 2.0,
                PARTICLE_SIZE,
                PARTICLE_SIZE,
            ],
            ctx.transform,
            g,
        );
    }
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }

    pub fn spawn_at(&mut self, grid_x: i32, grid_y: i32, color: Color) {
        // Convert grid position to pixel position (center of the block)
        let px = grid_x as f64 * BLOCK_SIZE + BLOCK_SIZE / 2.0;
        let py = grid_y as f64 * BLOCK_SIZE + BLOCK_SIZE / 2.0;

        for _ in 0..PARTICLE_COUNT {
            self.particles.push(Particle::new(px, py, color));
        }
    }

    pub fn update(&mut self, dt: f64) {
        for particle in &mut self.particles {
            particle.update(dt);
        }

        // Remove dead particles
        self.particles.retain(|p| p.is_alive());
    }

    pub fn draw(&self, ctx: &Context, g: &mut G2d) {
        for particle in &self.particles {
            particle.draw(ctx, g);
        }
    }
}
```

---

### src/physics.rs

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    #[allow(dead_code)]
    pub fn move_to_dir(&mut self, dir: Direction) {
        match dir {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}
```

---

### src/snake.rs

```rust
use std::collections::LinkedList;

use piston_window::{types::Color, Context, G2d};
use rand::Rng;

use crate::colors;
use crate::draw::*;
use crate::physics::{Direction, Position};

const INITIAL_SNAKE_TAIL_LENGTH: usize = 2;

pub struct Snake {
    direction: Direction,
    head: Position,
    tail: LinkedList<Position>,
    updated_tail_pos: bool,
    color: Color,
}

impl Snake {
    pub fn new(head: Position) -> Self {
        let (x, y) = (head.x, head.y);
        let mut tail = LinkedList::new();

        for i in 1..=INITIAL_SNAKE_TAIL_LENGTH {
            tail.push_back(Position { x, y: y - i as i32 });
        }

        Self {
            direction: Direction::Down,
            head: Position { x, y },
            tail,
            updated_tail_pos: false,
            color: colors::SNAKE,
        }
    }

    pub fn update(&mut self, _width: u32, _height: u32) {
        if !self.tail.is_empty() {
            self.tail.push_front(self.head.clone());
            self.tail.pop_back();
        }

        match self.direction {
            Direction::Up => self.head.y -= 1,
            Direction::Right => self.head.x += 1,
            Direction::Down => self.head.y += 1,
            Direction::Left => self.head.x -= 1,
        }

        // Wall wrapping removed - collision is now checked separately
        self.updated_tail_pos = true;
    }

    /// Check if the snake will hit a wall on the next move
    pub fn will_hit_wall(&self, width: u32, height: u32) -> bool {
        let next = self.next_head_pos();
        next.x < 0 || next.x >= width as i32 || next.y < 0 || next.y >= height as i32
    }

    pub fn draw(&self, ctx: &Context, g: &mut G2d) {
        for block in self.tail.iter() {
            draw_block(ctx, g, self.color, block);
        }

        draw_snake_head(ctx, g, self.color, &self.head, &self.direction);
    }

    pub fn set_dir(&mut self, dir: Direction) {
        if dir == self.direction.opposite() || !self.updated_tail_pos {
            return;
        }

        self.direction = dir;
        self.updated_tail_pos = false;
    }

    pub fn grow(&mut self) {
        let last = self.tail.back().cloned().unwrap_or(self.head.clone());
        self.tail.push_back(last);

        // 🎨 randomize color
        let mut rng = rand::thread_rng();
        self.color = [
            rng.gen_range(0.2..1.0),
            rng.gen_range(0.2..1.0),
            rng.gen_range(0.2..1.0),
            1.0,
        ];
    }

    pub fn get_head_pos(&self) -> &Position {
        &self.head
    }

    pub fn is_tail_overlapping(&self) -> bool {
        self.tail.iter().any(|pos| *pos == self.head)
    }

    fn next_head_pos(&self) -> Position {
        let mut pos = self.head.clone();

        match self.direction {
            Direction::Up => pos.y -= 1,
            Direction::Left => pos.x -= 1,
            Direction::Down => pos.y += 1,
            Direction::Right => pos.x += 1,
        }

        pos
    }
    pub fn will_tail_overlapp(&self) -> bool {
        let next = self.next_head_pos();

        self.tail.iter().any(|pos| *pos == next)
    }

    pub fn get_len(&self) -> usize {
        self.tail.len() - INITIAL_SNAKE_TAIL_LENGTH
    }
}
```
