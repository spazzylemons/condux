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

use std::{collections::HashMap, rc::Rc};

use crate::{assets::Asset, render::Mesh, vehicle::Model};

/// The garage manages vehicle models.
#[derive(Default)]
pub struct Garage {
    models: HashMap<String, Rc<Model>>,
}

impl Garage {
    /// Loads the hardcoded vehicle models.
    pub fn load_hardcoded(&mut self) {
        let mesh = Mesh::load(&mut Asset::load("mesh_vehicle.bin").unwrap()).unwrap();
        let model = Model {
            speed: 15.0,
            acceleration: 7.0,
            handling: 1.5,
            anti_drift: 12.0,
            mesh,
        };
        self.models.insert("default".into(), Rc::new(model));
    }

    /// Get a reference to a model by name.
    pub fn get(&self, name: &str) -> Option<Rc<Model>> {
        Some(self.models.get(name)?.clone())
    }
}
