use std::{slice::from_raw_parts, mem::zeroed, ffi::CString};

use crate::{bindings, linalg::Vector};

impl bindings::Asset {
    pub fn load(name: &str) -> Option<Self> {
        let mut result = unsafe {zeroed() };
        let name_convert = CString::new(name).unwrap();
        if !unsafe { bindings::asset_load(&mut result as *mut Self, name_convert.as_ptr()) } {
            None
        } else {
            Some(result)
        }
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let entry = unsafe { &*self.entry };
        let data = unsafe { from_raw_parts(entry.data, entry.size) };
        if self.index >= data.len() {
            return None;
        }
        let b = data[self.index] as u8;
        self.index += 1;
        Some(b)
    }

    pub fn read_fixed(&mut self) -> Option<f32> {
        let lo = self.read_byte()?;
        let hi = self.read_byte()?;
        Some(f32::from((u16::from(lo) | (u16::from(hi) << 8)) as i16) / 256.0)
    }

    pub fn read_vector(&mut self) -> Option<Vector> {
        let x = self.read_fixed()?;
        let y = self.read_fixed()?;
        let z = self.read_fixed()?;
        Some(Vector::new(x, y, z))
    }
}
