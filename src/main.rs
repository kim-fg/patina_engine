use patina_engine::prelude::*;

fn main() {
    let player = Actor::new("Player");
    println!("{}", player);
    let player_transform = player.transform();
    
    let player_model = Actor::new("Model");
    let model_transform = player_model.transform();
    model_transform.borrow_mut().set_parent(Some(player.transform()));
    player_transform.borrow_mut().add_child(model_transform.clone());
    

    let model_parent = model_transform.borrow().parent();
    println!("{:?}", model_parent);
}
