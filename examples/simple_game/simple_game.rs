use patina_engine::GameInstance;

#[derive(Default)]
pub struct SimpleGame {
    
}
impl SimpleGame {
    const TITLE: &str = "Simple Game";
}
impl GameInstance for SimpleGame {
    fn title(&self) -> String {
        SimpleGame::TITLE.into()
    }
    
    fn default_size(&self) -> (u32, u32) {
        (1280, 720)
    }
}