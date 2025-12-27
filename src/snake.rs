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
