mod gol;
use ggez::{ContextBuilder, event};

fn main() {
    let setup = ggez::conf::WindowSetup::default().title("Game of Life");
    let (mut ctx, event_loop) = ContextBuilder::new("Game of Life", "Ryan Fong")
        .window_setup(setup)
        .build()
        .expect("Failed to create ggez context!");
    let game_of_life = gol::GoL::new(&mut ctx);
    event::run(ctx, event_loop, game_of_life);    
}
