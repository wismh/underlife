use crate::resources::AssetError;
use crate::resources::types::config::ConfigAsset;

#[derive(Debug, Clone, Copy)]
pub struct PlayerConfig {
    pub spawn: SpawnConfig,
    pub movement: MovementConfig,
    pub head_bob: HeadBobConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct SpawnConfig {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MovementConfig {
    pub rotate_speed: f32,
    pub move_speed: f32,
    pub strafe_speed: f32,
    pub mouse_sensitivity: f32,
    pub plane_scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct HeadBobConfig {
    pub walk_phase_base: f32,
    pub walk_phase_speed_scale: f32,
    pub walk_horiz_phase_offset: f32,
    pub walk_vert_base: f32,
    pub walk_vert_speed_scale: f32,
    pub walk_horiz_scale: f32,
    pub walk_horiz_speed_base: f32,
    pub walk_horiz_speed_scale: f32,
    pub idle_phase_speed: f32,
    pub idle_vert_amplitude: f32,
    pub idle_horiz_amplitude: f32,
    pub idle_horiz_phase_scale: f32,
    pub smooth_y_idle: f32,
    pub smooth_y_walk: f32,
    pub smooth_x_idle: f32,
    pub smooth_x_walk: f32,
}

impl PlayerConfig {
    pub fn load(config: &ConfigAsset) -> Result<Self, AssetError> {
        let spawn = config.section("spawn")?;
        let movement = config.section("movement")?;
        let head_bob = config.section("head_bob")?;
        let path = config.path();

        Ok(Self {
            spawn: SpawnConfig {
                x: get_f32(path, spawn, "x")?,
                y: get_f32(path, spawn, "y")?,
                yaw: get_f32(path, spawn, "yaw")?,
            },
            movement: MovementConfig {
                rotate_speed: get_f32(path, movement, "rotate_speed")?,
                move_speed: get_f32(path, movement, "move_speed")?,
                strafe_speed: get_f32(path, movement, "strafe_speed")?,
                mouse_sensitivity: get_f32(path, movement, "mouse_sensitivity")?,
                plane_scale: get_f32(path, movement, "plane_scale")?,
            },
            head_bob: HeadBobConfig {
                walk_phase_base: get_f32(path, head_bob, "walk_phase_base")?,
                walk_phase_speed_scale: get_f32(path, head_bob, "walk_phase_speed_scale")?,
                walk_horiz_phase_offset: get_f32(path, head_bob, "walk_horiz_phase_offset")?,
                walk_vert_base: get_f32(path, head_bob, "walk_vert_base")?,
                walk_vert_speed_scale: get_f32(path, head_bob, "walk_vert_speed_scale")?,
                walk_horiz_scale: get_f32(path, head_bob, "walk_horiz_scale")?,
                walk_horiz_speed_base: get_f32(path, head_bob, "walk_horiz_speed_base")?,
                walk_horiz_speed_scale: get_f32(path, head_bob, "walk_horiz_speed_scale")?,
                idle_phase_speed: get_f32(path, head_bob, "idle_phase_speed")?,
                idle_vert_amplitude: get_f32(path, head_bob, "idle_vert_amplitude")?,
                idle_horiz_amplitude: get_f32(path, head_bob, "idle_horiz_amplitude")?,
                idle_horiz_phase_scale: get_f32(path, head_bob, "idle_horiz_phase_scale")?,
                smooth_y_idle: get_f32(path, head_bob, "smooth_y_idle")?,
                smooth_y_walk: get_f32(path, head_bob, "smooth_y_walk")?,
                smooth_x_idle: get_f32(path, head_bob, "smooth_x_idle")?,
                smooth_x_walk: get_f32(path, head_bob, "smooth_x_walk")?,
            },
        })
    }
}

fn get_f32(
    path: &str,
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<f32, AssetError> {
    let Some(value) = table.get(key) else {
        return Err(AssetError::InvalidConfig {
            path: path.to_string(),
            reason: format!("missing key `{key}`"),
        });
    };

    value
        .as_float()
        .map(|number| number as f32)
        .or_else(|| value.as_integer().map(|number| number as f32))
        .ok_or_else(|| AssetError::InvalidConfig {
            path: path.to_string(),
            reason: format!("`{key}` must be a number"),
        })
}
