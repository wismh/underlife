use std::path::Path;

use crate::resources::asset::{Asset, AssetError};

#[derive(Debug, Clone)]
pub struct MapAsset {
    pub width: u32,
    pub height: u32,
    cells: Vec<u8>,
    collision: Vec<u8>,
}

impl MapAsset {
    pub fn cell(&self, x: u32, y: u32) -> u8 {
        self.cells[(y * self.width + x) as usize]
    }

    pub fn is_wall(&self, x: f32, y: f32) -> bool {
        let mx = x.floor() as i32;
        let my = y.floor() as i32;
        if mx < 0 || my < 0 {
            return true;
        }
        let mx = mx as u32;
        let my = my as u32;
        if mx >= self.width || my >= self.height {
            return true;
        }
        self.cell(mx, my) != 0
    }

    pub fn cells_r8(&self) -> Vec<u8> {
        self.collision.clone()
    }

    pub fn width_f32(&self) -> f32 {
        self.width as f32
    }

    pub fn height_f32(&self) -> f32 {
        self.height as f32
    }
}

impl Asset for MapAsset {
    fn load(path: &Path) -> Result<Self, AssetError> {
        let text = std::fs::read_to_string(path).map_err(|source| AssetError::Io {
            path: path.display().to_string(),
            source,
        })?;

        let mut rows = Vec::new();
        for line in text.lines() {
            let line = line.trim_end();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            rows.push(parse_row(line));
        }

        if rows.is_empty() {
            return Err(AssetError::InvalidMap {
                path: path.display().to_string(),
                reason: "map file contains no rows".to_string(),
            });
        }

        let height = rows.len() as u32;
        let width = rows[0].len() as u32;
        if width == 0 {
            return Err(AssetError::InvalidMap {
                path: path.display().to_string(),
                reason: "map rows must not be empty".to_string(),
            });
        }

        for (index, row) in rows.iter().enumerate() {
            if row.len() as u32 != width {
                return Err(AssetError::InvalidMap {
                    path: path.display().to_string(),
                    reason: format!("row {index} has width {}, expected {width}", row.len()),
                });
            }
        }

        let mut cells = Vec::with_capacity((width * height) as usize);
        let mut collision = Vec::with_capacity((width * height) as usize);
        for row in rows {
            for cell in row {
                cells.push(cell);
                collision.push(if cell != 0 { 255 } else { 0 });
            }
        }

        Ok(Self {
            width,
            height,
            cells,
            collision,
        })
    }
}

fn parse_row(line: &str) -> Vec<u8> {
    line.chars()
        .map(|ch| match ch {
            '#' | '1' => 1,
            '.' | '0' | ' ' => 0,
            _ => 1,
        })
        .collect()
}
