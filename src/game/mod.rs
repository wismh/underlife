pub mod map;
pub mod player;

pub use map::{map_data, Map, MAP_H, MAP_W};
pub use player::{default_player, Player, MOVE_SPEED, ROTATE_SPEED};
