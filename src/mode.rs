use crate::{
    linalg::Vector,
    platform::{Buttons, Frame},
    render::Renderer,
};

/// A game mode.
pub trait Mode {
    /// Update the state.
    fn tick(&mut self, pressed: Buttons);

    /// Get the camera to render with.
    fn camera(&self, interp: f32) -> (Vector, Vector, Vector);

    /// Render this mode.
    fn render(&self, interp: f32, renderer: &Renderer, frame: &mut Frame);
}
