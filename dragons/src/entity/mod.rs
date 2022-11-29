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

#[derive(Debug)]
pub struct ComponentBundle {
    pub parent: Option<Parent>,
    pub transform: Option<Transform>,
    pub camera: Option<Camera>,
}

#[derive(Debug, Default)]
pub struct EntityBuilder {
    parent: Option<Parent>,
    transform: Option<Transform>,
    camera: Option<Camera>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parent(mut self, entity: EntityId) -> Self {
        self.parent = Some(Parent(entity));
        self
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn camera(mut self, camera: Camera) -> Self {
        self.camera = Some(camera);
        self
    }

    pub fn build(self) -> ComponentBundle {
        ComponentBundle {
            parent: self.parent,
            transform: self.transform,
            camera: self.camera,
        }
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
