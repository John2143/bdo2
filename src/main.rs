use amethyst::{
    SimpleState,
    GameDataBuilder,
};

struct GameState;
impl SimpleState for GameState {}



#[tokio::main]
async fn main() -> Result<(), ()> {
    amethyst::start_logger(Default::default());

    let assets = amethyst::utils::application_root_dir().unwrap().join("assets");

    let game_data = GameDataBuilder::default();

    let mut game = amethyst::Application::new(assets, GameState, game_data).unwrap();

    game.run();

    Ok(())
}
