use nalgebra_glm::{Vec3, Vec4};
use rand::prelude::*;
use std::f32::consts::PI;
use crate::{Framebuffer, Uniforms};

pub struct Skybox {
    stars: Vec<Star>,
}

struct Star {
    position: Vec3,
    brightness: f32,
    size: u8,
}

impl Star {
    /// Crea una estrella con propiedades aleatorias
    pub fn new(radius: f32) -> Self {
        let mut rng = rand::thread_rng();
        let theta = rng.gen::<f32>() * 2.0 * PI; // Ángulo azimutal
        let phi = rng.gen::<f32>() * PI;         // Ángulo polar

        // Coordenadas cartesianas
        let x = radius * phi.sin() * theta.cos();
        let y = radius * phi.cos();
        let z = radius * phi.sin() * theta.sin();

        Star {
            position: Vec3::new(x, y, z),
            brightness: rng.gen::<f32>(), // Brillo entre 0.0 y 1.0
            size: rng.gen_range(1..=3),  // Tamaño entre 1 y 3 píxeles
        }
    }
}

impl Skybox {
    /// Genera un nuevo skybox con un número específico de estrellas
    pub fn new(star_count: usize, radius: f32) -> Self {
        let stars = (0..star_count)
            .map(|_| Star::new(radius))
            .collect();
        Skybox { stars }
    }

    /// Renderiza el skybox al framebuffer
    pub fn render_sb(&self, framebuffer: &mut Framebuffer, uniforms: &Uniforms, camera_position: Vec3) {
        for star in &self.stars {
            // Posición relativa a la cámara
            let position = star.position + camera_position;

            // Proyectar al espacio de pantalla
            let pos_vec4 = Vec4::new(position.x, position.y, position.z, 1.0);
            let projected = uniforms.projection_matrix * uniforms.view_matrix * pos_vec4;

            // División perspectiva
            if projected.w <= 0.0 { continue; }
            let ndc = projected / projected.w;

            // Transformar a coordenadas de pantalla
            let screen_pos = uniforms.viewport_matrix * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);

            // Verificar si está en frente de la cámara
            if screen_pos.z < 0.0 { continue; }

            let x = screen_pos.x as usize;
            let y = screen_pos.y as usize;

            // Verificar si está dentro de los límites del framebuffer
            if x < framebuffer.width && y < framebuffer.height {
                // Calcular el color basado en el brillo
                let intensity = (star.brightness * 255.0) as u8;
                let color = (intensity as u32) << 16 | (intensity as u32) << 8 | intensity as u32;

                framebuffer.set_current_color(color);

                // Renderizar según el tamaño de la estrella
                match star.size {
                    1 => framebuffer.point(x, y, 1000.0),
                    2 => {
                        framebuffer.point(x, y, 1000.0);
                        framebuffer.point(x + 1, y, 1000.0);
                        framebuffer.point(x, y + 1, 1000.0);
                        framebuffer.point(x + 1, y + 1, 1000.0);
                    }
                    3 => {
                        framebuffer.point(x, y, 1000.0);
                        framebuffer.point(x - 1, y, 1000.0);
                        framebuffer.point(x + 1, y, 1000.0);
                        framebuffer.point(x, y - 1, 1000.0);
                        framebuffer.point(x, y + 1, 1000.0);
                    }
                    _ => {}
                }
            }
        }
    }
}
