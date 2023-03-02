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

use std::{collections::HashMap, error::Error, ffi::CString};

use super::{Buttons, Controls, Platform};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::style)]
#[allow(clippy::pedantic)]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

macro_rules! gl_resource_wrapper {
    (single $name:ident $ctor:ident $dtor:ident $($arg:ident : $t:ty),*) => {
        struct $name {
            id: gl::types::GLuint,
        }

        impl $name {
            fn new($($arg: $t),*) -> Self {
                Self {
                    id: unsafe { gl::$ctor($($arg),*) }
                }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    gl::$dtor(self.id);
                }
            }
        }
    };

    (batch $name:ident $ctor:ident $dtor:ident) => {
        struct $name {
            id: gl::types::GLuint,
        }

        impl $name {
            fn new() -> Self {
                let mut id: gl::types::GLuint = 0;
                unsafe {
                    gl::$ctor(1, &mut id);
                }
                Self { id }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    gl::$dtor(1, &self.id);
                }
            }
        }
    }
}

gl_resource_wrapper! { single Shader CreateShader DeleteShader ty: gl::types::GLenum }
gl_resource_wrapper! { single Program CreateProgram DeleteProgram }
gl_resource_wrapper! { batch VAO GenVertexArrays DeleteVertexArrays }
gl_resource_wrapper! { batch VBO GenBuffers DeleteBuffers }
gl_resource_wrapper! { batch Framebuffer GenFramebuffers DeleteFramebuffers }
gl_resource_wrapper! { batch Texture GenTextures DeleteTextures }

/// Wraps a program, VAO, and VBO together and renders them as one unit.
struct RenderUnit {
    program: Program,
    vao: VAO,
    vbo: VBO,
    num_attributes: usize,
    uniforms: HashMap<&'static str, gl::types::GLint>,
}

impl RenderUnit {
    fn new(
        vertex_source: &str,
        fragment_source: &str,
        num_attributes: usize,
        uniform_list: &[&'static str],
    ) -> Result<Self, Box<dyn Error>> {
        let program = Program::create(vertex_source, fragment_source)?;
        let vao = VAO::new();
        let vbo = VBO::new();
        // create attributes
        let stride = 2 * num_attributes * std::mem::size_of::<f32>();
        unsafe {
            gl::BindVertexArray(vao.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.id);
        }
        for i in 0..num_attributes {
            unsafe {
                gl::VertexAttribPointer(
                    i as _,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    stride as _,
                    std::mem::transmute(2 * i * std::mem::size_of::<f32>()),
                );
                gl::EnableVertexAttribArray(i as _);
            }
        }
        let mut uniforms = HashMap::new();
        for &uniform in uniform_list {
            let cstr = CString::new(uniform)?;
            let location = unsafe { gl::GetUniformLocation(program.id, cstr.as_ptr()) };
            if location == -1 {
                return Err(format!("unknown uniform: {uniform}").into());
            }
            uniforms.insert(uniform, location);
        }
        Ok(Self {
            program,
            vao,
            vbo,
            num_attributes,
            uniforms,
        })
    }

    fn start<'a>(&self, points: &'a [f32]) -> RenderUnitBuilder<'_, 'a> {
        RenderUnitBuilder {
            render_unit: self,
            clear: false,
            points,
            primitives: vec![],
            texture: None,
            framebuffer: None,
            uniforms: vec![],
        }
    }
}

struct RenderUnitBuilder<'a, 'b> {
    /// A reference to the render unit to use.
    render_unit: &'a RenderUnit,
    /// If true, we will clear the screen first.
    clear: bool,
    /// List of points to render to.
    points: &'b [f32],
    /// Primitives to use for the points.
    primitives: Vec<gl::types::GLenum>,
    /// The texture to bind when rendering.
    texture: Option<&'b Texture>,
    /// The framebuffer to render to.
    framebuffer: Option<(u16, u16, &'b Framebuffer, &'b Texture)>,
    /// The uniforms
    uniforms: Vec<(gl::types::GLint, f32, f32)>,
}

impl<'a, 'b> RenderUnitBuilder<'a, 'b> {
    fn clear(mut self) -> Self {
        self.clear = true;
        self
    }

    fn primitive(mut self, primitive: gl::types::GLenum) -> Self {
        self.primitives.push(primitive);
        self
    }

    fn texture(mut self, texture: &'b Texture) -> Self {
        self.texture = Some(texture);
        self
    }

    fn framebuffer(
        mut self,
        width: u16,
        height: u16,
        framebuffer: &'b Framebuffer,
        texture: &'b Texture,
    ) -> Self {
        self.framebuffer = Some((width, height, framebuffer, texture));
        self
    }

    fn uniform(mut self, name: &'static str, x: f32, y: f32) -> Self {
        let uniform = *self
            .render_unit
            .uniforms
            .get(name)
            .expect("invalid uniform");
        self.uniforms.push((uniform, x, y));
        self
    }

    fn render(self) {
        if let Some((width, height, framebuffer, texture)) = self.framebuffer {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.id);
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                // allocate texture to be size of screen
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as _,
                    width.into(),
                    height.into(),
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null(),
                );
                // set filters - we shouldn't need them because the framebuffer will always be the screen size
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
                // attach to framebuffer
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    texture.id,
                    0,
                );
            }
        } else {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
        };

        unsafe {
            if let Some(texture) = self.texture {
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
            } else {
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            gl::BindVertexArray(self.render_unit.vao.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.render_unit.vbo.id);
            gl::UseProgram(self.render_unit.program.id);

            if self.clear {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
        }

        for (uniform, x, y) in self.uniforms {
            unsafe {
                gl::Uniform2f(uniform, x, y);
            }
        }

        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * self.points.len()) as _,
                self.points.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
        }
        for primitive in self.primitives {
            unsafe {
                gl::DrawArrays(
                    primitive,
                    0,
                    (self.points.len() / (self.render_unit.num_attributes * 2)) as _,
                );
            }
        }
    }
}

impl Shader {
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

impl Program {
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

    fn create(vertex: &str, fragment: &str) -> Result<Self, String> {
        let vertex_shader = Shader::new(gl::VERTEX_SHADER);
        vertex_shader.source(vertex);
        vertex_shader.compile()?;
        let fragment_shader = Shader::new(gl::FRAGMENT_SHADER);
        fragment_shader.source(fragment);
        fragment_shader.compile()?;
        let result = Self::new();
        result.attach(&vertex_shader);
        result.attach(&fragment_shader);
        result.link()?;
        Ok(result)
    }
}

pub struct SdlPlatform {
    controller_ctx: sdl2::GameControllerSubsystem,
    event_pump: sdl2::EventPump,
    window: sdl2::video::Window,
    _gl_ctx: sdl2::video::GLContext,

    width: u16,
    height: u16,

    should_run: bool,

    keyboard_buttons: Buttons,

    controller: Option<sdl2::controller::GameController>,
    /// Scroll wheel this frame.
    scroll_wheel: i32,

    /// Buffered points.
    points: Vec<f32>,
    /// Render unit for lines.
    lines_unit: RenderUnit,
    /// Render unit for framebuffer.
    framebuffer_unit: RenderUnit,
    /// The texture to use on the framebuffer.
    texture: Texture,
    /// The framebuffer to use.
    framebuffer: Framebuffer,
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

// (x, y) and (u, v) of the framebuffer quad
static QUAD_VERTICES: [f32; 16] = [
    -1.0, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 0.0,
];

macro_rules! shader {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/shader/",
            $name,
            ".glsl"
        ))
    };
}

impl SdlPlatform {
    pub fn get_mouse(&self) -> sdl2::mouse::MouseState {
        self.event_pump.mouse_state()
    }

    pub fn get_scroll_wheel(&self) -> i32 {
        self.scroll_wheel
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

        let lines_unit =
            RenderUnit::new(shader!("vertex"), shader!("fragment"), 1, &["viewport"]).unwrap();
        let framebuffer_unit = RenderUnit::new(
            shader!("vertex_framebuffer"),
            shader!("fragment_framebuffer"),
            2,
            &[],
        )
        .unwrap();

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        Self {
            controller_ctx,
            event_pump,
            window,
            _gl_ctx: gl_ctx,

            width: preferred_width,
            height: preferred_height,

            should_run: true,
            keyboard_buttons: Buttons::empty(),

            controller: None,

            scroll_wheel: 0,

            points: vec![],
            lines_unit,
            framebuffer_unit,
            texture: Texture::new(),
            framebuffer: Framebuffer::new(),
        }
    }

    fn should_run(&self) -> bool {
        self.should_run
    }

    fn buffer_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.points.push(x0);
        self.points.push(y0);
        self.points.push(x1);
        self.points.push(y1);
    }

    fn end_frame(&mut self) {
        self.lines_unit
            .start(&self.points)
            .clear()
            .primitive(gl::LINES)
            .primitive(gl::POINTS)
            .framebuffer(self.width, self.height, &self.framebuffer, &self.texture)
            .uniform("viewport", self.width.into(), self.height.into())
            .render();
        self.points.clear();
        // two-pass gaussian
        self.framebuffer_unit
            .start(&QUAD_VERTICES)
            .primitive(gl::TRIANGLE_STRIP)
            .texture(&self.texture)
            .render();
        // swap buffers
        self.window.gl_swap_window();
        // accept events
        self.scroll_wheel = 0;
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

                Event::MouseWheel { y, direction, .. } => {
                    self.scroll_wheel = y;
                    if direction == sdl2::mouse::MouseWheelDirection::Flipped {
                        self.scroll_wheel = -self.scroll_wheel;
                    }
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
