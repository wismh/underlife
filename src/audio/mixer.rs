use std::time::Duration;

/// Placeholder hooks for future mixer features (ducking, EQ, reverb, snapshots).
#[derive(Debug, Default)]
pub struct MixerState {
    pub active_snapshot: Option<MixerSnapshotId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MixerSnapshotId(pub u32);

impl MixerState {
    pub fn apply_snapshot(&mut self, id: MixerSnapshotId) {
        self.active_snapshot = Some(id);
    }

    pub fn clear_snapshot(&mut self) {
        self.active_snapshot = None;
    }

    pub fn set_ducking(&mut self, _amount: f32, _fade: Duration) {}

    pub fn set_reverb_send(&mut self, _amount: f32) {}

    pub fn set_eq_low(&mut self, _gain_db: f32) {}
}
