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

    pub fn update(&mut self, dt: f32, bob_speed: f32) {
        self.walk_phase += dt * (4.2 + bob_speed * 1.7);
        self.idle_phase += dt * 0.9;

        let walk_sin = self.walk_phase.sin();
        let walk_vert = walk_sin * walk_sin.abs();
        let walk_horiz = (self.walk_phase + 1.1).sin();

        let walk_target_y = walk_vert * (9.0 + bob_speed * 2.4);
        let walk_target_x = walk_horiz * 0.013 * (0.7 + bob_speed * 0.08);

        let idle_target_y = self.idle_phase.sin() * 7.5;
        let idle_target_x = (self.idle_phase * 0.6).sin() * 0.009;

        let walk_blend = smoothstep(0.0, 1.0, bob_speed / MOVE_SPEED);
        let target_y = lerp(idle_target_y, walk_target_y, walk_blend);
        let target_x = lerp(idle_target_x, walk_target_x, walk_blend);

        let smooth_y = lerp(6.5, 15.0, walk_blend);
        let smooth_x = lerp(5.0, 8.5, walk_blend);
        self.offset_y = approach(self.offset_y, target_y, dt, smooth_y);
        self.offset_x = approach(self.offset_x, target_x, dt, smooth_x);
    }
}

impl Default for HeadBob {
    fn default() -> Self {
        Self::new()
    }
}

const MOVE_SPEED: f32 = 3.5;

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
