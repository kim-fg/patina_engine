mod simple_game;

extern crate patina_engine;

use patina_engine::GameInstance;
use simple_game::SimpleGame;

fn main() {
    let game = SimpleGame::default();
    println!("Running {:?} in Patina Engine", game.title());
}