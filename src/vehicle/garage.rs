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

use std::collections::HashMap;

use crate::{assets::Asset, render::Mesh, vehicle::Model};

/// The garage manages vehicle models.
#[derive(Default)]
pub struct Garage {
    by_name: HashMap<String, u16>,
    by_id: Vec<Model>,
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
        self.load("default".into(), model).unwrap();
    }

    /// Try to load or replace a model with the given name.
    pub fn load(&mut self, name: String, value: Model) -> Result<(), String> {
        if let Some(id) = self.get_id(&name) {
            // if name already used, replace that model
            self.by_id[usize::from(id)] = value;
        } else {
            // attempt to allocate
            let new_id = match u16::try_from(self.by_id.len()) {
                Ok(v) => v,
                Err(_) => return Err("too many models in garage".into()),
            };
            self.by_id.push(value);
            self.by_name.insert(name, new_id);
        }
        Ok(())
    }

    /// Get a model ID by name.
    pub fn get_id(&self, name: &str) -> Option<u16> {
        Some(*self.by_name.get(name)?)
    }

    /// Get a model from its ID.
    pub fn get_model(&self, id: u16) -> Option<&Model> {
        self.by_id.get(usize::from(id))
    }
}
