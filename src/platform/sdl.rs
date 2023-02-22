//! Condux - an antigravity racing game
//! Copyright (C) 2023 spazzylemons
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <http://www.gnu.org/licenses/>.

use sdl2::event::Event;

use std::time::Instant;

use super::{Buttons, Controls, Line, Platform};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::style)]
#[allow(clippy::pedantic)]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct SdlPlatform {
    controller_ctx: sdl2::GameControllerSubsystem,
    event_pump: sdl2::EventPump,
    window: sdl2::video::Window,
    _gl_ctx: sdl2::video::GLContext,

    epoch: Instant,

    width: u16,
    height: u16,

    should_run: bool,

    keyboard_buttons: Buttons,

    controller: Option<sdl2::controller::GameController>,
}

static KEYBOARD_MAPPING: [sdl2::keyboard::Keycode; 7] = [
    sdl2::keyboard::Keycode::Up,
    sdl2::keyboard::Keycode::Down,
    sdl2::keyboard::Keycode::Left,
    sdl2::keyboard::Keycode::Right,
    sdl2::keyboard::Keycode::X,
    sdl2::keyboard::Keycode::Z,
    sdl2::keyboard::Keycode::Escape,
];

static BUTTON_MAPPING: [sdl2::controller::Button; 7] = [
    sdl2::controller::Button::DPadUp,
    sdl2::controller::Button::DPadDown,
    sdl2::controller::Button::DPadLeft,
    sdl2::controller::Button::DPadRight,
    sdl2::controller::Button::A,
    sdl2::controller::Button::B,
    sdl2::controller::Button::Start,
];

fn get_keycode_bitmask(keycode: sdl2::keyboard::Keycode) -> Buttons {
    for (i, k) in KEYBOARD_MAPPING.iter().enumerate() {
        if *k == keycode {
            return Buttons::from_bits(1 << i).unwrap();
        }
    }
    Buttons::empty()
}

impl SdlPlatform {
    fn point(&self, x: f32, y: f32) {
        // convert to [-1, 1]
        let width = f32::from(self.width());
        let height = f32::from(self.height());
        let x = (x / (width * 0.5)) + ((1.0 - width) / width);
        let y = -((y / (height * 0.5)) + ((1.0 - height) / height));
        unsafe {
            gl::Vertex2f(x, y);
        }
    }
}

impl Platform for SdlPlatform {
    fn init(preferred_width: u16, preferred_height: u16) -> Self {
        let ctx = sdl2::init().unwrap();
        let video = ctx.video().unwrap();
        let window = video
            .window("window", preferred_width.into(), preferred_height.into())
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .unwrap();
        gl::load_with(|s| video.gl_get_proc_address(s).cast());
        let gl_ctx = window.gl_create_context().unwrap();

        let controller_ctx = ctx.game_controller().unwrap();
        let event_pump = ctx.event_pump().unwrap();

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Color3f(1.0, 1.0, 1.0);
        }

        Self {
            controller_ctx,
            event_pump,
            window,
            _gl_ctx: gl_ctx,

            epoch: Instant::now(),

            width: preferred_width,
            height: preferred_height,

            should_run: true,
            keyboard_buttons: Buttons::empty(),

            controller: None,
        }
    }

    fn should_run(&self) -> bool {
        self.should_run
    }

    fn time_msec(&self) -> u64 {
        self.epoch
            .elapsed()
            .as_millis()
            .try_into()
            .expect("you've been running the game too long!")
    }

    fn end_frame(&mut self, lines: &[Line]) {
        unsafe {
            // clear screen
            gl::Clear(gl::COLOR_BUFFER_BIT);
            // begin drawing lines
            gl::Begin(gl::LINES);
        }
        // draw the lines
        for ((x0, y0), (x1, y1)) in lines {
            self.point(*x0, *y0);
            self.point(*x1, *y1);
        }
        // finish frame
        unsafe {
            gl::End();
        }
        // swap buffers
        self.window.gl_swap_window();
        // accept events
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::Close => {
                        // window close
                        self.should_run = false;
                    }

                    sdl2::event::WindowEvent::Resized(x, y) => unsafe {
                        gl::Viewport(0, 0, x, y);
                    },

                    _ => {}
                },

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    self.keyboard_buttons |= get_keycode_bitmask(keycode);
                }

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    self.keyboard_buttons &= !get_keycode_bitmask(keycode);
                }

                _ => {}
            }
        }
        // update dimensions
        let (width, height) = self.window.drawable_size();
        self.width = u16::try_from(width).unwrap();
        self.height = u16::try_from(height).unwrap();
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn poll(&mut self) -> Controls {
        if let Some(controller) = &self.controller {
            // close controller if not attached
            if !controller.attached() {
                self.controller = None;
            }
        }
        // attempt to open a controller if not already opened
        if self.controller.is_none() {
            match self.controller_ctx.num_joysticks() {
                Ok(n) => {
                    for i in 0..n {
                        if self.controller_ctx.is_game_controller(i) {
                            match self.controller_ctx.open(i) {
                                Ok(controller) => {
                                    self.controller = Some(controller);
                                }

                                Err(e) => {
                                    eprintln!("failed to connect controller: {e}");
                                }
                            }
                        }
                    }
                }

                Err(e) => {
                    eprintln!("failed to query joysticks: {e}");
                }
            }
        }
        let mut buttons = self.keyboard_buttons;
        let mut steering = 0.0;
        if let Some(controller) = &self.controller {
            for (i, b) in BUTTON_MAPPING.iter().enumerate() {
                if controller.button(*b) {
                    buttons |= Buttons::from_bits(1 << i).unwrap();
                }
            }
            let mut axis = controller.axis(sdl2::controller::Axis::LeftX);
            if axis == -32768 {
                axis = -32767;
            }
            steering = f32::from(axis) / 32767.0;
        } else {
            // if no controller connected, use keyboard steering
            if buttons.contains(Buttons::LEFT) {
                steering = -1.0;
            } else if buttons.contains(Buttons::RIGHT) {
                steering = 1.0;
            }
        }
        Controls { buttons, steering }
    }
}
