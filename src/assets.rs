use include_dir::{Dir, include_dir};

use crate::linalg::Vector;

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

pub struct Asset {
    contents: &'static [u8],
    index: usize,
}

impl Asset {
    #[must_use]
    pub fn load(name: &str) -> Option<Self> {
        let contents = ASSETS.get_file(name)?.contents();
        Some(Self { contents, index: 0 })
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.index >= self.contents.len() {
            return None;
        }
        let b = self.contents[self.index];
        self.index += 1;
        Some(b)
    }

    pub fn read_fixed(&mut self) -> Option<f32> {
        let lo = self.read_byte()?;
        let hi = self.read_byte()?;
        Some(f32::from(i16::from(lo) | (i16::from(hi) << 8)) / 256.0)
    }

    pub fn read_vector(&mut self) -> Option<Vector> {
        let x = self.read_fixed()?;
        let y = self.read_fixed()?;
        let z = self.read_fixed()?;
        Some(Vector::new(x, y, z))
    }
}
