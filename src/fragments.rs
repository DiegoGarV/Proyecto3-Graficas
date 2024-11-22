use nalgebra_glm::{Vec3, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct Fragments {
    pub position: Vec2,
    pub depth: f32,
    pub normal: Vec3,
    pub intensity: f32,
    pub vertex_pos: Vec3,
}

impl Fragments {
    pub fn new(
        position: Vec2,
        depth: f32,
        normal: Vec3,
        intensity: f32,
        vertex_pos: Vec3,
    ) -> Self {
        Fragments {
            position,
            depth,
            normal,
            intensity,
            vertex_pos,
        }
    }
}