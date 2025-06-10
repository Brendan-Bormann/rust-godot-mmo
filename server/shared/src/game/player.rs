use crate::game::vector::{Vector2, Vector3};
use bitcode::{Decode, Encode};

#[derive(Decode, Encode, Debug, Clone, PartialEq)]
pub struct Player {
    pub id: String,
    pub username: String,
    pub position: Vector2,
    pub input_direction: Vector2,
    pub rotation: f32, // radians
    pub speed: f32,
}

impl Player {
    pub fn new(id: String, username: String) -> Player {
        Player {
            id,
            username,
            position: Vector2::new(0.0, 0.0),
            input_direction: Vector2::zero(),
            rotation: 0.0,
            speed: 5.0,
        }
    }

    pub fn from_string(player_string: String) -> Player {
        let player_parts: Vec<&str> = player_string.split(';').collect();

        let id: String = player_parts[0].parse().unwrap();
        let username: String = player_parts[1].into();

        let position: Vector2 = Vector2::from_string(player_parts[2].into());
        let input_direction: Vector2 = Vector2::from_string(player_parts[4].into());

        let rotation: f32 = player_parts[5].parse().unwrap();
        let speed: f32 = player_parts[6].parse().unwrap();

        Player {
            id,
            username,
            position,
            input_direction,
            rotation,
            speed,
        }
    }

    pub fn player_vec_to_string(player_vec: Vec<Player>) -> String {
        let players: Vec<String> = player_vec.iter().map(|p| p.to_string()).collect();
        players.join("+")
    }
}

impl ToString for Player {
    fn to_string(&self) -> String {
        format!(
            "{};{};{};{};{:.2};{}",
            self.id,
            self.username,
            self.position.to_string(),
            self.input_direction.to_string(),
            self.rotation,
            self.speed
        )
    }
}

impl Player {
    pub fn get_target_pos(&mut self, delta_time: f64) -> Vector2 {
        let mut delta = self.input_direction.clone().normalize();
        delta.multiply(self.speed * delta_time as f32);

        Vector2 {
            x: self.position.x + delta.x,
            y: self.position.y + delta.y,
        }
    }

    pub fn move_to(&mut self, position: Vector2) {
        self.position = position;
    }
}
