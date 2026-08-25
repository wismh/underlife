use crate::game::config::{HeadBobConfig, MovementConfig};

#[derive(Debug, Clone, Copy)]
pub struct HeadBob {
    walk_phase: f32,
    idle_phase: f32,
    pub offset_y: f32,
    pub offset_x: f32,
}

impl HeadBob {
    pub fn new() -> Self {
        Self {
            walk_phase: 0.0,
            idle_phase: 0.0,
            offset_y: 0.0,
            offset_x: 0.0,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        bob_speed: f32,
        cfg: HeadBobConfig,
        movement: MovementConfig,
    ) {
        self.walk_phase += dt * (cfg.walk_phase_base + bob_speed * cfg.walk_phase_speed_scale);
        self.idle_phase += dt * cfg.idle_phase_speed;

        let walk_sin = self.walk_phase.sin();
        let walk_vert = walk_sin * walk_sin.abs();
        let walk_horiz = (self.walk_phase + cfg.walk_horiz_phase_offset).sin();

        let walk_target_y =
            walk_vert * (cfg.walk_vert_base + bob_speed * cfg.walk_vert_speed_scale);
        let walk_target_x = walk_horiz
            * cfg.walk_horiz_scale
            * (cfg.walk_horiz_speed_base + bob_speed * cfg.walk_horiz_speed_scale);

        let idle_target_y = self.idle_phase.sin() * cfg.idle_vert_amplitude;
        let idle_target_x =
            (self.idle_phase * cfg.idle_horiz_phase_scale).sin() * cfg.idle_horiz_amplitude;

        let walk_blend = smoothstep(0.0, 1.0, bob_speed / movement.move_speed);
        let target_y = lerp(idle_target_y, walk_target_y, walk_blend);
        let target_x = lerp(idle_target_x, walk_target_x, walk_blend);

        let smooth_y = lerp(cfg.smooth_y_idle, cfg.smooth_y_walk, walk_blend);
        let smooth_x = lerp(cfg.smooth_x_idle, cfg.smooth_x_walk, walk_blend);
        self.offset_y = approach(self.offset_y, target_y, dt, smooth_y);
        self.offset_x = approach(self.offset_x, target_x, dt, smooth_x);
    }
}

impl Default for HeadBob {
    fn default() -> Self {
        Self::new()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn approach(current: f32, target: f32, dt: f32, rate: f32) -> f32 {
    current + (target - current) * (1.0 - (-dt * rate).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::config::{HeadBobConfig, MovementConfig};

    fn configs() -> (HeadBobConfig, MovementConfig) {
        (
            HeadBobConfig {
                walk_phase_base: 4.2,
                walk_phase_speed_scale: 1.7,
                walk_horiz_phase_offset: 1.1,
                walk_vert_base: 9.0,
                walk_vert_speed_scale: 2.4,
                walk_horiz_scale: 0.013,
                walk_horiz_speed_base: 0.7,
                walk_horiz_speed_scale: 0.08,
                idle_phase_speed: 0.9,
                idle_vert_amplitude: 7.5,
                idle_horiz_amplitude: 0.009,
                idle_horiz_phase_scale: 0.6,
                smooth_y_idle: 6.5,
                smooth_y_walk: 15.0,
                smooth_x_idle: 5.0,
                smooth_x_walk: 8.5,
            },
            MovementConfig {
                rotate_speed: 2.5,
                move_speed: 3.0,
                strafe_speed: 1.5,
                mouse_sensitivity: 0.0015,
                plane_scale: 0.66,
            },
        )
    }

    #[test]
    fn head_bob_offset_changes_when_bob_speed_positive() {
        let (head_bob_cfg, movement) = configs();
        let mut bob = HeadBob::new();
        let start_x = bob.offset_x;
        let start_y = bob.offset_y;

        for _ in 0..45 {
            bob.update(1.0 / 60.0, movement.move_speed, head_bob_cfg, movement);
        }

        assert!(
            (bob.offset_x - start_x).abs() > 1e-4 || (bob.offset_y - start_y).abs() > 1e-4,
            "expected head-bob offset to change, got ({}, {})",
            bob.offset_x,
            bob.offset_y
        );
    }
}
