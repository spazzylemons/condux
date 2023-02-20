use ctru::{gfx::{RawFrameBuffer, Screen}, prelude::*};

use std::error::Error;

use super::{Buttons, Controls, Platform};

static KEY_MAPPING: [ctru::services::hid::KeyPad; 7] = [
    ctru::services::hid::KeyPad::KEY_UP,
    ctru::services::hid::KeyPad::KEY_DOWN,
    ctru::services::hid::KeyPad::KEY_LEFT,
    ctru::services::hid::KeyPad::KEY_RIGHT,
    ctru::services::hid::KeyPad::KEY_B,
    ctru::services::hid::KeyPad::KEY_A,
    ctru::services::hid::KeyPad::KEY_START,
];

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod citro2d_sys {
    extern "C" {
        pub fn C2D_Init(maxObjects: usize) -> bool;
        pub fn C2D_Fini();
        pub fn C2D_Prepare();
        pub fn C2D_CreateScreenTarget(screen: ctru_sys::gfxScreen_t, side: ctru_sys::gfx3dSide_t) -> *mut citro3d_sys::C3D_RenderTarget;
        pub fn C2D_TargetClear(target: *mut citro3d_sys::C3D_RenderTarget, color: u32);
        pub fn C2D_Flush();
        pub fn C2D_SceneSize(width: u32, height: u32, tilt: bool);
        pub fn C2D_DrawLine(x0: f32, y0: f32, clr0: u32, x1: f32, y1: f32, clr1: u32, thickness: f32, depth: f32) -> bool;
    }

    pub const C2D_DEFAULT_MAX_OBJECTS: u32 = 4096;

    #[inline]
    pub unsafe fn C2D_SceneTarget(target: *mut citro3d_sys::C3D_RenderTarget) {
        let target = &*target;
        // the boolean not was not in the original function, but this doesn't seem to work without it???
        C2D_SceneSize(target.frameBuf.width.into(), target.frameBuf.height.into(), !target.linked);
    }

    #[inline]
    pub unsafe fn C2D_SceneBegin(target: *mut citro3d_sys::C3D_RenderTarget) {
        C2D_Flush();
        citro3d_sys::C3D_FrameDrawOn(target);
        C2D_SceneTarget(target);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
    cx: f32,
    cy: f32,
    cz: f32,
}

pub struct CitroPlatform {
    hid: Hid,
    gfx: Gfx,
    apt: Apt,
    target: *mut citro3d_sys::C3D_RenderTarget,
}

impl Drop for CitroPlatform {
    fn drop(&mut self) {
        unsafe {
            // clean up citro stuff
            citro3d_sys::C3D_RenderTargetDelete(self.target);
            citro2d_sys::C2D_Fini();
            citro3d_sys::C3D_Fini();
        }
    }
}

impl Platform for CitroPlatform {
    fn init(_preferred_width: u16, _preferred_height: u16) -> Result<Self, Box<dyn Error>> {
        ctru::use_panic_handler();

        let hid = Hid::init()?;
        let gfx = Gfx::init()?;
        let apt = Apt::init()?;

        let target = unsafe {
            // initialize citro3d
            citro3d_sys::C3D_Init(citro3d_sys::C3D_DEFAULT_CMDBUF_SIZE as usize);
            // initialize citro2d
            citro2d_sys::C2D_Init(citro2d_sys::C2D_DEFAULT_MAX_OBJECTS as usize);
            // prepare citro2d
            citro2d_sys::C2D_Prepare();
            // get the scene target
            citro2d_sys::C2D_CreateScreenTarget(ctru_sys::GFX_TOP, ctru_sys::GFX_LEFT)
        };

        Ok(Self {
            hid,
            gfx,
            apt,
            target,
        })
    }

    fn should_run(&self) -> bool {
        self.apt.main_loop()
    }

    fn time_msec(&self) -> u64 {
        unsafe {
            ctru_sys::osGetTime()
        }
    }

    fn end_frame(&mut self, lines: &[((f32, f32), (f32, f32))]) {
        unsafe {
            citro3d_sys::C3D_FrameBegin(citro3d_sys::C3D_FRAME_SYNCDRAW as u8);
            citro2d_sys::C2D_TargetClear(self.target, 0xff_00_00_00);
            citro2d_sys::C2D_SceneBegin(self.target);
            for ((x0, y0), (x1, y1)) in lines {
                citro2d_sys::C2D_DrawLine(*x0, *y0, 0xffffffff, *x1, *y1, 0xffffffff, 1.0, 0.0);
            }
            citro3d_sys::C3D_FrameEnd(0);
        }
    }

    fn width(&self) -> u16 {
        400
    }

    fn height(&self) -> u16 {
        240
    }

    fn poll(&mut self) -> Controls {
        self.hid.scan_input();
        let held = self.hid.keys_held();
        let mut buttons = Buttons::empty();
        for (i, k) in KEY_MAPPING.iter().enumerate() {
            if held.contains(*k) {
                buttons |= Buttons::from_bits(1 << i).unwrap();
            }
        }
        let mut circle_pos = ctru::services::hid::CirclePosition::default();
        let (x, _) = circle_pos.get();
        let steering = (f32::from(x) / 156.0).clamp(-1.0, 1.0);
        Controls { buttons, steering }
    }
}
