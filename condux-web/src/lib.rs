use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    condux::run_game();
}
