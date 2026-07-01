pub const MAP_W: usize = 16;
pub const MAP_H: usize = 16;

pub fn map_data() -> [[u8; MAP_W]; MAP_H] {
    [
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1],
        [1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1],
        [1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
        [1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ]
}

pub struct Map {
    grid: [[u8; MAP_W]; MAP_H],
    cells: Vec<u8>,
}

impl Map {
    pub fn new(grid: [[u8; MAP_W]; MAP_H]) -> Self {
        let cells = map_pixels(&grid);
        Self { grid, cells }
    }

    pub fn grid(&self) -> &[[u8; MAP_W]; MAP_H] {
        &self.grid
    }

    pub fn cells(&self) -> &[u8] {
        &self.cells
    }
}

pub fn is_wall(map: &[[u8; MAP_W]; MAP_H], x: f32, y: f32) -> bool {
    let mx = x.floor() as i32;
    let my = y.floor() as i32;
    if mx < 0 || my < 0 || mx as usize >= MAP_W || my as usize >= MAP_H {
        return true;
    }
    map[my as usize][mx as usize] != 0
}

fn map_pixels(map: &[[u8; MAP_W]; MAP_H]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(MAP_W * MAP_H);
    for row in map {
        for cell in row {
            pixels.push(if *cell != 0 { 255 } else { 0 });
        }
    }
    pixels
}
