use std::sync::Arc;

use specs::{Component, VecStorage};

use crate::engine::model;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Mesh {
    pub id: usize,
    pub mesh: Arc<model::Mesh>,
}

impl Mesh {
    pub fn new(id: usize, mesh: Arc<model::Mesh>) -> Self {
        Self {
            id,
            mesh
        }
    }
}