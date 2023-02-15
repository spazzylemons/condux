#[repr(C)]
pub struct AssetEntry {
    pub name: *const u8,
    pub size: usize,
    pub data: *const u8,
}

#[repr(C)]
pub struct Asset {
    entry: *const AssetEntry,
    index: usize,
}

extern "C" {
    pub fn asset_load(asset: *mut Asset, name: *const u8) -> bool;
    pub fn asset_read_byte(asset: *mut Asset, b: *mut u8) -> bool;
    pub fn asset_read_fixed(asset: *mut Asset, f: *mut f32) -> bool;
    pub fn asset_read_vec(asset: *mut Asset, v: *mut f32) -> bool;
}