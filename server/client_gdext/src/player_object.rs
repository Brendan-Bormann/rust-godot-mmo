use godot::classes::Node3D;
use godot::prelude::*;
use shared::game::player::Player;
use shared::util::math::lerp;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct PlayerObject {
    #[var]
    pub id: GString,
    #[var]
    pub username: GString,
    #[var]
    pub network_position: Vector3,
    pub base: Base<Node3D>,
}

#[godot_api]
impl PlayerObject {
    pub fn network_set_player(&mut self, player: &Player) {
        let pos = self.base().get_position();

        self.id = player.id.clone().into();
        self.username = player.username.clone().into();
        let updated_position = Vector3 {
            x: player.position.x,
            y: 0.0,
            z: player.position.y,
        };

        self.network_position = updated_position;

        if pos.x == 0.0 && pos.y == 0.0 && pos.z == 0.0 {
            self.base_mut().set_position(updated_position);
        }
    }
}
