use std::{cell::Cell, rc::Rc};

use super::{Buttons, Controls, Line, Platform};

use wasm_bindgen::prelude::*;

pub struct WebPlatform {
    /// buttons pressed by keyboard
    keyboard_buttons: Rc<Cell<Buttons>>,
    /// canvas to draw to
    canvas: web_sys::HtmlCanvasElement,
    /// 2d context for the canvas
    ctx: web_sys::CanvasRenderingContext2d,
    /// Performance to read time from
    performance: web_sys::Performance,
    /// Navigator to read gamepads from
    navigator: web_sys::Navigator,
    /// element to show for gamepad mapping note
    gamepad_mapping_note: web_sys::Element,
    /// Reference to keydown event listener
    _key_down: Closure<dyn Fn(web_sys::KeyboardEvent)>,
    /// Reference to keyup event listener
    _key_up: Closure<dyn Fn(web_sys::KeyboardEvent)>,
}

fn is_pressed(button: &JsValue) -> bool {
    if let Some(button) = button.dyn_ref::<web_sys::GamepadButton>() {
        button.pressed()
    } else {
        false
    }
}

static KEYBOARD_MAPPING: [&str; 7] = [
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "x",
    "z",
    "Escape",
];

static BUTTON_MAPPING: [i32; 7] = [12, 13, 14, 15, 1, 0, 9];

fn get_keycode_bitmask(keycode: &str) -> Buttons {
    for (i, k) in KEYBOARD_MAPPING.iter().enumerate() {
        if *k == keycode {
            return Buttons::from_bits(1 << i).unwrap();
        }
    }
    Buttons::empty()
}

impl Platform for WebPlatform {
    fn init(preferred_width: u16, preferred_height: u16) -> Self {
        // get window
        let window = web_sys::window().unwrap();
        // get document
        let document = window.document().unwrap();
        // get canvas element
        let canvas = document
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        // set canvas dimensions
        canvas.set_width(preferred_width.into());
        canvas.set_height(preferred_height.into());
        // get the canvas context
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        // get the performance object
        let performance = window.performance().unwrap();
        // get the navigator object
        let navigator = window.navigator();
        // get the mapping note element
        let gamepad_mapping_note = document.get_element_by_id("gamepad-mapping-note").unwrap();
        // create keyboard buttons reference
        let keyboard_buttons = Rc::new(Cell::new(Buttons::empty()));
        // create keydown listener
        let keyboard_buttons_clone = keyboard_buttons.clone();
        let key_down =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                let new_val = keyboard_buttons_clone.get() | get_keycode_bitmask(&event.key());
                keyboard_buttons_clone.set(new_val);
            });
        // create keyup listener
        let keyboard_buttons_clone = keyboard_buttons.clone();
        let key_up =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                let new_val = keyboard_buttons_clone.get() & !get_keycode_bitmask(&event.key());
                keyboard_buttons_clone.set(new_val);
            });
        // register listeners
        window
            .add_event_listener_with_callback("keydown", key_down.as_ref().unchecked_ref())
            .unwrap();
        window
            .add_event_listener_with_callback("keyup", key_up.as_ref().unchecked_ref())
            .unwrap();
        // return platform object
        Self {
            keyboard_buttons,
            canvas,
            ctx,
            performance,
            navigator,
            gamepad_mapping_note,

            _key_down: key_down,
            _key_up: key_up,
        }
    }

    fn should_run(&self) -> bool {
        true
    }

    fn time_msec(&self) -> u64 {
        self.performance.now() as _
    }

    fn end_frame(&mut self, lines: &[Line]) {
        // clear screen with black
        self.ctx
            .set_fill_style(&wasm_bindgen::JsValue::from_str("black"));
        self.ctx.fill_rect(
            0.0,
            0.0,
            self.canvas.width().into(),
            self.canvas.height().into(),
        );
        // set line style
        self.ctx.set_line_width(1.0);
        self.ctx
            .set_stroke_style(&wasm_bindgen::JsValue::from_str("white"));
        for ((x0, y0), (x1, y1)) in lines {
            self.ctx.begin_path();
            // add 0.5 for less blurry text
            self.ctx.move_to((*x0 + 0.5).into(), (*y0 + 0.5).into());
            self.ctx.line_to((*x1 + 0.5).into(), (*y1 + 0.5).into());
            self.ctx.stroke();
        }
    }

    fn width(&self) -> u16 {
        self.canvas.width() as _
    }

    fn height(&self) -> u16 {
        self.canvas.height() as _
    }

    fn poll(&mut self) -> Controls {
        let mut buttons = self.keyboard_buttons.get();
        let mut steering = 0.0;

        let mut current_gamepad = None;
        let mut found_gamepad = false;
        for gamepad in self.navigator.get_gamepads().unwrap().iter() {
            if let Ok(gamepad) = gamepad.dyn_into::<web_sys::Gamepad>() {
                found_gamepad = true;
                if gamepad.mapping() == web_sys::GamepadMappingType::Standard {
                    current_gamepad = Some(gamepad);
                }
            }
        }

        self.gamepad_mapping_note
            .set_attribute("hidden", "")
            .unwrap();
        if let Some(gamepad) = current_gamepad {
            for (i, b) in BUTTON_MAPPING.iter().enumerate() {
                if is_pressed(&gamepad.buttons().at(*b)) {
                    buttons |= Buttons::from_bits(1 << i).unwrap();
                }
            }
            if let Some(axis) = gamepad.axes().at(0).as_f64() {
                steering = axis as f32;
            }
        } else {
            if found_gamepad {
                self.gamepad_mapping_note
                    .remove_attribute("hidden")
                    .unwrap();
            }

            if buttons.contains(Buttons::LEFT) {
                steering = -1.0;
            } else if buttons.contains(Buttons::RIGHT) {
                steering = 1.0;
            }
        }

        Controls { buttons, steering }
    }
}
