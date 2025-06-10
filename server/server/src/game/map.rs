use shared::game::vector::Vector2;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Clone, Copy)]
enum TileType {
    Empty,
    Walkable,
}

#[derive(Clone)]
pub struct Tile {
    tile_type: TileType,
}

impl Tile {
    pub fn new(walkable: bool) -> Tile {
        Tile {
            tile_type: if walkable {
                TileType::Walkable
            } else {
                TileType::Empty
            },
        }
    }

    pub fn is_walkable(&self) -> bool {
        matches!(self.tile_type, TileType::Walkable)
    }
}

type TilePosition = (i32, i32);

pub struct Map {
    pub tiles: HashMap<TilePosition, Tile>,
}

impl Map {
    pub fn new(filename: String) -> Self {
        let content = read_to_string(filename).unwrap();
        let mut tiles = HashMap::new();

        for (y, line) in content.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let tile = match ch {
                    '#' => Tile::new(false),
                    '.' => Tile::new(true),
                    _ => Tile::new(false),
                };
                tiles.insert((x as i32, y as i32), tile);
            }
        }

        Map { tiles }
    }

    pub fn set_tile(&mut self, position: Vector2, tile: Tile) {
        let x = position.x.floor() as i32;
        let y = position.y.floor() as i32;
        self.tiles.insert((x, y), tile);
    }

    pub fn get_tile(&self, position: Vector2) -> Option<&Tile> {
        let x = position.x.floor() as i32;
        let y = position.y.floor() as i32;
        self.tiles.get(&(x, y))
    }

    pub fn is_valid(&self, position: Vector2) -> bool {
        self.get_tile(position)
            .map(|tile| tile.is_walkable())
            .unwrap_or(false)
    }

    pub fn move_to(&self, start: Vector2, dest: Vector2) -> Result<(), Vector2> {
        if !self.is_valid(start) {
            return Err(start);
        }

        if self.is_valid(dest) {
            return Ok(());
        }

        let steps = 10;
        let delta = (dest - start) / steps as f32;
        let mut current = start;

        for _ in 0..steps {
            let next = current + delta;
            if !self.is_valid(next) {
                return Err(current);
            }
            current = next;
        }

        Err(current)
    }
}
