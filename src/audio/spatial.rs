use glam::Vec3;

use crate::audio::api::SpatialParams;
use crate::audio::volume::linear_to_decibels;

#[derive(Debug, Clone, Copy)]
pub struct ListenerState {
    pub position: Vec3,
    pub forward: Vec3,
}

impl Default for ListenerState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: Vec3::NEG_Z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpatialAttenuation {
    pub volume_linear: f32,
    pub pan: f32,
}

pub fn spatial_attenuation(listener: ListenerState, params: SpatialParams) -> SpatialAttenuation {
    let offset = params.position - listener.position;
    let distance = offset.length();
    let max_distance = params.max_distance.max(0.01);

    let volume_linear = (1.0 - (distance / max_distance).clamp(0.0, 1.0)).clamp(0.0, 1.0);

    let forward = listener.forward.normalize_or_zero();
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    let pan = if right.length_squared() > 0.0 {
        (offset.normalize_or_zero().dot(right) * 0.5 + 0.5).clamp(0.0, 1.0)
    } else {
        0.5
    };

    let _ = linear_to_decibels(volume_linear);
    SpatialAttenuation {
        volume_linear,
        pan,
    }
}
