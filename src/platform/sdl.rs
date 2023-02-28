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

use std::{ffi::CString, time::Instant};

use super::{Buttons, Controls, Platform};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::style)]
#[allow(clippy::pedantic)]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    fn new(ty: gl::types::GLenum) -> Self {
        Self {
            id: unsafe { gl::CreateShader(ty) },
        }
    }

    fn source(&self, source: &str) {
        let strings = [source.as_ptr() as *const gl::types::GLchar];
        let lengths = [source.len() as gl::types::GLint];
        unsafe {
            gl::ShaderSource(self.id, 1, strings.as_ptr(), lengths.as_ptr());
        }
    }

    fn compile(&self) -> Result<(), String> {
        unsafe {
            gl::CompileShader(self.id);
            // check success of the compilation
            let mut success: gl::types::GLint = 0;
            gl::GetShaderiv(
                self.id,
                gl::COMPILE_STATUS,
                &mut success as *mut gl::types::GLint,
            );
            if success == 0 {
                let mut length: gl::types::GLint = 0;
                gl::GetShaderiv(
                    self.id,
                    gl::INFO_LOG_LENGTH,
                    &mut length as *mut gl::types::GLint,
                );
                let mut buffer = vec![0u8; length as usize];
                gl::GetShaderInfoLog(
                    self.id,
                    length,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut gl::types::GLchar,
                );
                // drop null terminator
                buffer.pop();
                Err(String::from_utf8(buffer).unwrap_or_else(|_| String::from("GL error")))
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

struct Program {
    id: gl::types::GLuint,
}

impl Program {
    fn new() -> Self {
        Self {
            id: unsafe { gl::CreateProgram() },
        }
    }

    fn attach(&self, shader: &Shader) {
        unsafe {
            gl::AttachShader(self.id, shader.id);
        }
    }

    fn link(&self) -> Result<(), String> {
        unsafe {
            gl::LinkProgram(self.id);
            // check success of the compilation
            let mut success: gl::types::GLint = 0;
            gl::GetProgramiv(
                self.id,
                gl::LINK_STATUS,
                &mut success as *mut gl::types::GLint,
            );
            if success == 0 {
                let mut length: gl::types::GLint = 0;
                gl::GetProgramiv(
                    self.id,
                    gl::INFO_LOG_LENGTH,
                    &mut length as *mut gl::types::GLint,
                );
                let mut buffer = vec![0u8; length as usize];
                gl::GetProgramInfoLog(
                    self.id,
                    length,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut gl::types::GLchar,
                );
                // drop null terminator
                buffer.pop();
                Err(String::from_utf8(buffer).unwrap_or_else(|_| String::from("GL error")))
            } else {
                Ok(())
            }
        }
    }

    fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    fn get_uniform(&self, name: &str) -> Option<Uniform> {
        if let Ok(cstr) = CString::new(name) {
            let location = unsafe { gl::GetUniformLocation(self.id, cstr.as_ptr()) };
            if location == -1 {
                None
            } else {
                Some(Uniform { location })
            }
        } else {
            None
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

struct VAO {
    id: gl::types::GLuint,
}

impl VAO {
    fn new() -> Self {
        let mut id: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        Self { id }
    }

    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    fn enable(index: gl::types::GLuint) {
        unsafe {
            gl::EnableVertexAttribArray(index);
        }
    }

    fn attrib_ptr(
        index: gl::types::GLuint,
        size: gl::types::GLint,
        ty: gl::types::GLenum,
        normalized: bool,
        stride: gl::types::GLsizei,
        offset: usize,
    ) {
        unsafe {
            gl::VertexAttribPointer(
                index,
                size,
                ty,
                normalized.into(),
                stride,
                std::mem::transmute(offset),
            );
        }
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

struct VBO {
    id: gl::types::GLuint,
}

impl VBO {
    fn new() -> Self {
        let mut id: gl::types::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Self { id }
    }

    fn bind(&self, target: gl::types::GLenum) {
        unsafe {
            gl::BindBuffer(target, self.id);
        }
    }

    fn data<T>(target: gl::types::GLenum, ptr: &[T], usage: gl::types::GLenum) {
        unsafe {
            gl::BufferData(
                target,
                (std::mem::size_of::<T>() * ptr.len()) as _,
                ptr.as_ptr() as *const std::ffi::c_void,
                usage,
            );
        }
    }
}

impl Drop for VBO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

struct Uniform {
    location: gl::types::GLint,
}

impl Uniform {
    fn vec2(&self, x: f32, y: f32) {
        unsafe {
            gl::Uniform2f(self.location, x, y);
        }
    }
}

fn draw_arrays(mode: gl::types::GLenum, start: gl::types::GLsizei, count: gl::types::GLsizei) {
    unsafe {
        gl::DrawArrays(mode, start, count);
    }
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

    /// Buffered points.
    points: Vec<f32>,
    /// Viewport shader uniform.
    viewport: Uniform,
    /// Active OpenGL shader program.
    _program: Program,
    /// Active vertex array object.
    _vao: VAO,
    /// Active vertex buffer object.
    _vbo: VBO,
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

        let vertex = Shader::new(gl::VERTEX_SHADER);
        vertex.source(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/shader/vertex.glsl"
        )));
        vertex.compile().unwrap();
        let fragment = Shader::new(gl::FRAGMENT_SHADER);
        fragment.source(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/shader/fragment.glsl"
        )));
        fragment.compile().unwrap();

        let program = Program::new();
        program.attach(&vertex);
        program.attach(&fragment);
        program.link().unwrap();
        program.use_program();

        let vao = VAO::new();
        let vbo = VBO::new();

        vao.bind();
        vbo.bind(gl::ARRAY_BUFFER);

        VAO::attrib_ptr(0, 2, gl::FLOAT, false, 0, 0);
        VAO::enable(0);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
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

            points: vec![],
            viewport: program.get_uniform("viewport").unwrap(),
            _program: program,
            _vao: vao,
            _vbo: vbo,
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

    fn buffer_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.points.push(x0);
        self.points.push(y0);
        self.points.push(x1);
        self.points.push(y1);
    }

    fn end_frame(&mut self) {
        unsafe {
            // clear screen
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // send viewport to shader
        self.viewport.vec2(self.width.into(), self.height.into());
        // load points into buffer
        VBO::data(gl::ARRAY_BUFFER, &self.points, gl::STATIC_DRAW);
        let num_points = self.points.len() / 2;
        // clear local point buffer
        self.points.clear();
        // use array buffer to draw lines
        draw_arrays(gl::LINES, 0, num_points as _);
        // use it again to connect them
        draw_arrays(gl::POINTS, 0, num_points as _);
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
