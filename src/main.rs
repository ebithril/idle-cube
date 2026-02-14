mod cube;
mod game;
mod moves;

use game::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Idle Cube".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    game.run().await;
}
