use ggez::{event, conf, ContextBuilder, GameResult};
use ggez::conf::FullscreenType;
use Turrets::MainState;

fn main() -> GameResult {
    // Initialize the game context and window
    let cb = ContextBuilder::new("Turrets", "jnbrauer")
        .window_setup(conf::WindowSetup::default().title("Turrets"))
        .window_mode(conf::WindowMode::default().fullscreen_type(FullscreenType::Windowed));

    let (ctx, events_loop) = &mut cb.build()?;

    // Initialize the game state
    let game = &mut MainState::new(ctx);
    // Start the game
    return event::run(ctx, events_loop, game);
}
