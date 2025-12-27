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
