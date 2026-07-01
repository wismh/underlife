use glam::Vec2;

use crate::resources::types::map::MapAsset;

pub struct Player {
    pub pos: Vec2,
    pub dir: Vec2,
    pub plane: Vec2,
}

impl Player {
    pub fn new(x: f32, y: f32, yaw: f32) -> Self {
        let dir = Vec2::new(yaw.cos(), yaw.sin());
        let plane = Vec2::new(-dir.y, dir.x) * 0.66;
        Self {
            pos: Vec2::new(x, y),
            dir,
            plane,
        }
    }

    pub fn rotate(&mut self, angle: f32) {
        let cos = angle.cos();
        let sin = angle.sin();
        self.dir = Vec2::new(
            self.dir.x * cos - self.dir.y * sin,
            self.dir.x * sin + self.dir.y * cos,
        );
        self.plane = Vec2::new(-self.dir.y, self.dir.x) * 0.66;
    }

    pub fn move_relative(
        &mut self,
        map: &MapAsset,
        forward: f32,
        strafe: f32,
        forward_speed: f32,
        strafe_speed: f32,
    ) {
        let velocity =
            self.dir * forward * forward_speed + Vec2::new(-self.dir.y, self.dir.x) * strafe * strafe_speed;

        let new_x = self.pos.x + velocity.x;
        if !map.is_wall(new_x, self.pos.y) {
            self.pos.x = new_x;
        }

        let new_y = self.pos.y + velocity.y;
        if !map.is_wall(self.pos.x, new_y) {
            self.pos.y = new_y;
        }

        self.pos.x = self.pos.x.clamp(0.25, map.width_f32() - 1.25);
        self.pos.y = self.pos.y.clamp(0.25, map.height_f32() - 1.25);
    }
}

pub const ROTATE_SPEED: f32 = 2.5;
pub const MOVE_SPEED: f32 = 3.5;
pub const STRAFE_SPEED: f32 = 2.0;
pub const MOUSE_SENSITIVITY: f32 = 0.0015;

pub fn default_player() -> Player {
    Player::new(2.5, 2.5, 0.0)
}
