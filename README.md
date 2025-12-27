# rsnake - An implementation of classic snake in rust

This game was built using the [piston_window](https://github.com/PistonDevelopers/piston_window) window wrapper.

<img src="./assets/snake.gif" style="width: 100%;" />

## Download the game
I use arch linux and endeavourOS. I have no idea what im doing, good luck!

```bash
# run the game
cargo run

# build a binary & execute it
cargo build --release
./target/release/rsnake
```

## Keymap
- **W/A/S/D** or **Up/Left/Down/Right** - Controll snake direction.
- **R** - Restart the game.
- **ESC** - Quit the game.

## Known Flaws / ToDo List
- **R** has no effect yet.
- There should be an endscreen with restart / quit buttons.
- The fruit and snake could possibly spawn at the same coordinate / fruits can spawn inside the tail of the snake.
