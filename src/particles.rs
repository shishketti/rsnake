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
