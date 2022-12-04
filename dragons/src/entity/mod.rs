use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use nalgebra::{UnitQuaternion, Vector3};

pub type EntityId = u32;

#[derive(Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            scale: Vector3::repeat(1.0f32),
            rotation: UnitQuaternion::identity(),
        }
    }
}

impl Transform {
    pub fn translate(mut self, offset: Vector3<f32>) -> Self {
        self.position += offset;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = Vector3::repeat(1.0).scale(scale);
        self
    }
}

#[derive(Debug)]
pub struct Parent(pub EntityId);

#[derive(Debug)]
pub struct Camera {
    pub fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::FRAC_PI_3,
        }
    }
}

// simplified implementation of entity component system

#[derive(Debug, Default)]
pub struct ComponentBundle {
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl ComponentBundle {
    pub fn add_component<T: Any>(&mut self, component: T) -> Option<T> {
        self.components
            .insert(TypeId::of::<T>(), Box::new(component))
            .map(|x| *x.downcast::<T>().unwrap())
    }

    pub fn component<T: Any>(&self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|x| x.downcast_ref())
            .flatten()
    }

    pub fn component_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .map(|x| x.downcast_mut())
            .flatten()
    }

    pub fn has<T: Any>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }
}

#[derive(Debug, Default)]
pub struct World {
    entities: Vec<(EntityId, ComponentBundle)>,
}

impl World {
    pub fn add(&mut self, bundle: ComponentBundle) -> EntityId {
        let entity_id = self.entities.last().map(|x| *&x.0).unwrap_or_default() + 1;
        self.entities.push((entity_id, bundle));
        entity_id
    }
}
