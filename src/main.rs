mod game;
mod position;
mod direction;
mod game_state;
mod vec2_pool;
mod spatial_hash;
mod texture_cache;
mod input_buffer;

use macroquad::prelude::*;
use game::Game;

#[macroquad::main("🐍 Ilon")]
async fn main() {
    let mut game = Game::new();
    game.initialize().await;

    loop {
        game.handle_input();
        game.update();
        game.draw();

        next_frame().await
    }
}