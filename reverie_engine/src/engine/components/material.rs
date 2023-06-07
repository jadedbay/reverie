use std::sync::Arc;

use serde::Serialize;
use specs::{Component, VecStorage};

use crate::engine::{gpu::Gpu, registry::Registry, material::Material};

use super::{ComponentDefault, TypeName};

#[derive(Clone, Component, Serialize)]
#[storage(VecStorage)]
pub struct MaterialComponent {
    pub id: usize,
    #[serde(skip)]
    pub material: Arc<Gpu<Material>>
}

impl MaterialComponent {
    pub fn new(id: usize, registry: &mut Registry) -> Self {
        let material = registry.get_material(id).unwrap();

        Self {
            id,
            material,
        }
    }
}

impl ComponentDefault for MaterialComponent {
    fn default(_device: &wgpu::Device, registry: &mut Registry) -> Self {
        let material = registry.get_material(1).unwrap();

        Self {
            id: 1,
            material,
        }
    }
}

impl TypeName for MaterialComponent {
    fn type_name() -> &'static str {
        "material"
    }
}