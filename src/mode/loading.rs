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

#[cfg(not(target_arch = "wasm32"))]
use std::thread::JoinHandle;

use crate::render::graph::RenderGraph;

use super::{GlobalGameData, Mode};

/// A mode that allows potentially blocking or long-running processes to run
/// without blocking the game thread.
/// TODO WASM support - for WASM we just let the operation block for now
pub struct LoadingMode<T> {
    #[cfg(not(target_arch = "wasm32"))]
    thread: JoinHandle<T>,
    #[cfg(target_arch = "wasm32")]
    function: Box<dyn FnOnce() -> T + Send + 'static>,
}

impl<T> LoadingMode<T>
where
    T: Send + 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                thread: std::thread::spawn(f),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                function: Box::new(f),
            }
        }
    }
}

fn render_mono(graph: &mut RenderGraph, height: u16) {
    graph.text(
        16.0,
        f32::from(height) - 16.0 - (6.0 * 4.0),
        4.0,
        String::from("Loading..."),
    );
}

impl<T> Mode for LoadingMode<T>
where
    T: Mode + 'static,
{
    fn tick(self: Box<Self>, _data: &GlobalGameData) -> Box<dyn Mode> {
        #[cfg(not(target_arch = "wasm32"))]
        if self.thread.is_finished() {
            // thread is done loading, switch to the newly loaded mode
            Box::new(self.thread.join().unwrap())
        } else {
            // still loading
            self
        }
        #[cfg(target_arch = "wasm32")]
        // blocks until completion on wasm
        Box::new((self.function)())
    }

    fn render(
        &self,
        _interp: f32,
        _data: &GlobalGameData,
        graph: &mut RenderGraph,
        _width: u16,
        height: u16,
    ) {
        render_mono(graph, height);
    }
}
