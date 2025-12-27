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
