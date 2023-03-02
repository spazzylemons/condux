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

use std::{cell::Cell, rc::Rc};

use crate::render::context::Line2d;

use super::{Buttons, Controls, Platform};

use wasm_bindgen::prelude::*;

struct TouchElement {
    /// The last read touch average, or None if the last event had no touches.
    touch_position: Rc<Cell<Option<(f64, f64)>>>,
    /// The event listener function.
    _callback: Closure<dyn Fn(web_sys::TouchEvent)>,
}

impl TouchElement {
    fn new(id: &str, document: &web_sys::Document) -> Self {
        // get the element by id
        let element = document.get_element_by_id(id).unwrap();
        // create a touch position cell
        let touch_position = Rc::new(Cell::new(None));
        // create a callback
        let touch_position_clone = touch_position.clone();
        let element_clone = element.clone();
        let callback =
            Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
                // don't do zoom/scroll/select on this element
                event.prevent_default();
                // find touch location
                let list = event.touches();
                let length = list.length();
                if length != 0 {
                    // get the bounding rectangle
                    let rect = element_clone.get_bounding_client_rect();
                    let rect_x = rect.x();
                    let rect_y = rect.y();
                    let rect_w_adjust = 2.0 / (rect.width() - 1.0);
                    let rect_h_adjust = 2.0 / (rect.height() - 1.0);
                    let mut result_x = 0.0;
                    let mut result_y = 0.0;
                    for i in 0..length {
                        let touch = list.item(i).unwrap();
                        result_x += f64::from(touch.client_x()) - rect_x;
                        result_y += f64::from(touch.client_y()) - rect_y;
                    }
                    let n = f64::from(length);
                    result_x /= n;
                    result_y /= n;
                    // recenter to [-1, 1]
                    result_x = (rect_w_adjust * result_x - 1.0).clamp(-1.0, 1.0);
                    result_y = -(rect_h_adjust * result_y - 1.0).clamp(-1.0, 1.0);
                    touch_position_clone.set(Some((result_x, result_y)));
                } else {
                    touch_position_clone.set(None);
                }
            });
        // add event listeners to element
        element
            .add_event_listener_with_callback("touchstart", callback.as_ref().unchecked_ref())
            .unwrap();
        element
            .add_event_listener_with_callback("touchmove", callback.as_ref().unchecked_ref())
            .unwrap();
        element
            .add_event_listener_with_callback("touchend", callback.as_ref().unchecked_ref())
            .unwrap();
        // return object
        Self {
            touch_position,
            _callback: callback,
        }
    }
}

pub struct WebPlatform {
    /// buttons pressed by keyboard
    keyboard_buttons: Rc<Cell<Buttons>>,
    /// canvas to draw to
    canvas: web_sys::HtmlCanvasElement,
    /// 2d context for the canvas
    ctx: web_sys::CanvasRenderingContext2d,
    /// Navigator to read gamepads from
    navigator: web_sys::Navigator,
    /// Virtual analog stick
    virtual_analog: TouchElement,
    /// Set to true when mobile pause button pressed
    pause_press: Rc<Cell<bool>>,
    /// element to show for gamepad mapping note
    gamepad_mapping_note: web_sys::Element,
    /// Lines to draw.
    lines: Vec<Line2d>,
    /// Reference to keydown event listener
    _key_down: Closure<dyn Fn(web_sys::KeyboardEvent)>,
    /// Reference to keyup event listener
    _key_up: Closure<dyn Fn(web_sys::KeyboardEvent)>,
    /// Reference to onclick event listener
    _on_click: Closure<dyn Fn()>,
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
                let bitmask = get_keycode_bitmask(&event.key());
                if !bitmask.is_empty() {
                    event.prevent_default();
                    let new_val = keyboard_buttons_clone.get() | bitmask;
                    keyboard_buttons_clone.set(new_val);
                }
            });
        // create keyup listener
        let keyboard_buttons_clone = keyboard_buttons.clone();
        let key_up =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
                let bitmask = get_keycode_bitmask(&event.key());
                if !bitmask.is_empty() {
                    event.prevent_default();
                    let new_val = keyboard_buttons_clone.get() & !bitmask;
                    keyboard_buttons_clone.set(new_val);
                }
            });
        // create pause button reference
        let pause_press = Rc::new(Cell::new(false));
        // create onclick listener
        let pause_press_clone = pause_press.clone();
        let on_click = Closure::<dyn Fn()>::new(move || pause_press_clone.set(true));
        // register listeners
        window
            .add_event_listener_with_callback("keydown", key_down.as_ref().unchecked_ref())
            .unwrap();
        window
            .add_event_listener_with_callback("keyup", key_up.as_ref().unchecked_ref())
            .unwrap();
        document
            .get_element_by_id("virtual-pause")
            .unwrap()
            .add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())
            .unwrap();
        // return platform object
        Self {
            keyboard_buttons,
            canvas,
            ctx,
            navigator,
            virtual_analog: TouchElement::new("virtual-analog", &document),
            pause_press,
            gamepad_mapping_note,
            lines: vec![],

            _key_down: key_down,
            _key_up: key_up,
            _on_click: on_click,
        }
    }

    fn buffer_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.lines.push(((x0, y0), (x1, y1)));
    }

    fn end_frame(&mut self) {
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
        for ((x0, y0), (x1, y1)) in &self.lines {
            self.ctx.begin_path();
            // add 0.5 for less blurry text
            self.ctx.move_to((*x0 + 0.5).into(), (*y0 + 0.5).into());
            self.ctx.line_to((*x1 + 0.5).into(), (*y1 + 0.5).into());
            self.ctx.stroke();
        }
        self.lines.clear();
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

        // does not support all buttons, but works for demo
        if let Some((x, y)) = self.virtual_analog.touch_position.get() {
            steering = x as _;
            if y <= -0.5 {
                buttons |= Buttons::BACK;
            } else if y >= 0.5 {
                buttons |= Buttons::OK;
            }
        }

        if self.pause_press.get() {
            buttons |= Buttons::PAUSE;
            self.pause_press.set(false);
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
