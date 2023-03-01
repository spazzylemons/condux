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

use std::sync::Arc;

use crate::linalg::{Mtx, Vector};

use super::{
    context::{Line2d, RenderContext, RenderContext3d, ScissorContext},
    Font, Mesh,
};

pub enum RenderNode {
    Line2d(Line2d),
    Text {
        x: f32,
        y: f32,
        scale: f32,
        text: String,
    },
    Graph3d(RenderGraph3d),
    Scissor {
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        graph: RenderGraph,
    },
}

impl RenderNode {
    pub fn render(self, font: &Font, ctx: &mut dyn RenderContext) {
        match self {
            Self::Line2d(((x0, y0), (x1, y1))) => ctx.line(x0, y0, x1, y1),
            Self::Text { x, y, scale, text } => font.write(ctx, x, y, scale, &text),
            Self::Graph3d(graph) => graph.render(ctx),
            Self::Scissor {
                min_x,
                min_y,
                max_x,
                max_y,
                graph,
            } => {
                let mut new_ctx = ScissorContext::new(ctx, min_x, min_y, max_x, max_y);
                graph.render(font, &mut new_ctx);
            }
        }
    }
}

#[derive(Default)]
pub struct RenderGraph {
    nodes: Vec<RenderNode>,
}

impl RenderGraph {
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.nodes.push(RenderNode::Line2d(((x0, y0), (x1, y1))));
    }

    pub fn text(&mut self, x: f32, y: f32, scale: f32, text: String) {
        self.nodes.push(RenderNode::Text { x, y, scale, text });
    }

    pub fn text_centered(&mut self, x: f32, y: f32, scale: f32, text: String) {
        let x = x - (Font::GLYPH_SPACING * (text.len() as f32) - 1.0) * (scale * 0.5);
        self.nodes.push(RenderNode::Text { x, y, scale, text });
    }

    pub fn scissor(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32, graph: RenderGraph) {
        self.nodes.push(RenderNode::Scissor {
            min_x,
            min_y,
            max_x,
            max_y,
            graph,
        });
    }

    pub fn graph_3d(&mut self, graph: RenderGraph3d) {
        self.nodes.push(RenderNode::Graph3d(graph));
    }

    pub fn render(self, font: &Font, ctx: &mut dyn RenderContext) {
        for node in self.nodes {
            node.render(font, ctx);
        }
    }
}

pub enum RenderNode3d {
    /// A list of lines to render without transformation. Used for spline rendering.
    Lines(Arc<Vec<(Vector, Vector)>>),
    /// A mesh.
    Mesh {
        translation: Vector,
        rotation: Mtx,
        mesh: Arc<Mesh>,
    },
}

impl RenderNode3d {
    pub fn render(self, ctx: &mut RenderContext3d) {
        match self {
            Self::Lines(lines) => {
                for (a, b) in lines.iter() {
                    ctx.line(*a, *b);
                }
            }
            Self::Mesh {
                translation,
                rotation,
                mesh,
            } => {
                mesh.render(ctx, translation, rotation);
            }
        }
    }
}

pub struct RenderGraph3d {
    eye: Vector,
    at: Vector,
    up: Vector,
    nodes: Vec<RenderNode3d>,
}

impl RenderGraph3d {
    pub fn new(eye: Vector, at: Vector, up: Vector) -> Self {
        Self {
            eye,
            at,
            up,
            nodes: vec![],
        }
    }

    pub fn lines(&mut self, lines: Arc<Vec<(Vector, Vector)>>) {
        self.nodes.push(RenderNode3d::Lines(lines));
    }

    pub fn mesh(&mut self, translation: Vector, rotation: Mtx, mesh: Arc<Mesh>) {
        self.nodes.push(RenderNode3d::Mesh {
            translation,
            rotation,
            mesh,
        });
    }

    pub fn render(self, ctx: &mut dyn RenderContext) {
        let mut ctx_3d = RenderContext3d::new(ctx, self.eye, self.at, self.up);
        for node in self.nodes {
            node.render(&mut ctx_3d);
        }
    }
}
