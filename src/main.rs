use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

mod triangle;
mod obj_loader;
mod color;
mod shaders;
mod framebuffer;
mod vertex;
mod fragments;
mod camera;

use vertex::Vertex;
use camera::Camera;
use obj_loader::Obj;
use framebuffer::Framebuffer;
use shaders::{fragment_shader, moon_position, vertex_shader, ShaderType};
use triangle::triangle;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    debug_mode: u32,
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}


fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render_rings(framebuffer: &mut Framebuffer, planet_position: Vec3, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    let ring_uniforms = Uniforms {
        model_matrix: create_model_matrix(planet_position, 2.0, Vec3::new(0.0, 0.0, 0.0)),
        view_matrix: uniforms.view_matrix,
        projection_matrix: uniforms.projection_matrix,
        viewport_matrix: uniforms.viewport_matrix,
        time: uniforms.time,
        debug_mode: uniforms.debug_mode,
    };
    let ring_shader = ShaderType::Ring;
    render(framebuffer, &ring_uniforms, vertex_array, &ring_shader);
}

fn moon_render(framebuffer: &mut Framebuffer, position: Vec3, time: u32, view_matrix: Mat4, projection_matrix: Mat4, viewport_matrix: Mat4, sphere_vertex_arrays: &[Vertex]) {
    let moon_pos = moon_position(time as f32, 1.3);
    let moon_uniforms = Uniforms {
        model_matrix: create_model_matrix(position + moon_pos, 0.5, Vec3::new(0.0, 0.0, 0.0)),
        view_matrix,
        projection_matrix,
        viewport_matrix,
        time,
        debug_mode: 0,
    };
    render(framebuffer, &moon_uniforms, sphere_vertex_arrays, &ShaderType::Moon);
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], current_shader: &ShaderType) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        let normal = (tri[1].position - tri[0].position).cross(&(tri[2].position - tri[0].position));
        let view_dir = tri[0].position - Vec3::new(0.0, 0.0, 0.0);
    
        if normal.dot(&view_dir) < 0.0 {
            continue;
        }
    
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Processing Stage
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            // Apply fragment shader
            let shaded_color = fragment_shader(&fragment, &uniforms, current_shader);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Planets Render",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x335555);

    // Configuración inicial de la cámara
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 25.0), // Alejamos la cámara para ver todos los planetas
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    // Cargar modelos 3D
    let sphere_loader = Obj::load("models/sphere.obj").expect("Failed to load sphere obj");
    let sphere_vertex_arrays = sphere_loader.get_vertex_array();

    let ring_loader = Obj::load("models/ring.obj").expect("Failed to load ring obj");
    let ring_vertex_array = ring_loader.get_vertex_array();

    let mut time = 0;

    let mut last_frame = Instant::now();

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let now = Instant::now();
        if now - last_frame < Duration::from_millis(16) {
            continue; // Limitar a ~60 FPS
        }
        last_frame = now;
        
        time += 1;

        handle_input(&window, &mut camera);

        framebuffer.clear();

        // Matrices comunes
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

        // Renderizar cada planeta con las escalas y distancias
        let planet_positions = vec![
            (Vec3::new(0.0, 0.0, 0.0), ShaderType::Sun, 10.0),
            (Vec3::new(10.0, 0.0, 0.0), ShaderType::VolcanicPlanet, 1.0),
            (Vec3::new(20.0, 0.0, 0.0), ShaderType::Earth, 1.5),
            (Vec3::new(30.0, 0.0, 0.0), ShaderType::RockyPlanet, 1.3),
            (Vec3::new(40.0, 0.0, 0.0), ShaderType::GasPlanet, 4.0),
            (Vec3::new(50.0, 0.0, 0.0), ShaderType::RingPlanet, 3.5),
            (Vec3::new(60.0, 0.0, 0.0), ShaderType::IcyPlanet, 0.8),
        ];      

        for (position, shader, scale) in planet_positions {           
            
            let uniforms = Uniforms {
                model_matrix: create_model_matrix(position, scale, Vec3::new(0.0, 0.0, 0.0)),
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time,
                debug_mode: 0,
            };

            // Renderizar planeta
            render(&mut framebuffer, &uniforms, &sphere_vertex_arrays, &shader);

            // Renderizar anillos o lunas si aplica
            match shader {
                ShaderType::RingPlanet => {
                    render_rings(&mut framebuffer, position, &uniforms, &ring_vertex_array);
                }
                ShaderType::RockyPlanet => {
                    moon_render(&mut framebuffer, position, time, view_matrix, projection_matrix, viewport_matrix, &sphere_vertex_arrays);
                }
                _ => {}
            }
            
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    }
}


fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 1.0;
    let rotation_speed = PI/50.0;
    let zoom_speed = 0.1;
   
    //  Camara orbital
    if window.is_key_down(Key::Left) {
      camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
      camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Up) {
      camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::Down) {
      camera.orbit(0.0, rotation_speed);
    }

    // Camara movimiento
    let mut movement = Vec3::new(0.0, 0.0, 0.0);
    if window.is_key_down(Key::A) {
      movement.x -= movement_speed;
    }
    if window.is_key_down(Key::D) {
      movement.x += movement_speed;
    }
    if window.is_key_down(Key::W) {
      movement.y += movement_speed;
    }
    if window.is_key_down(Key::S) {
      movement.y -= movement_speed;
    }
    if movement.magnitude() > 0.0 {
      camera.move_center(movement);
    }

    // Zoom
    if window.is_key_down(Key::M) {
      camera.zoom(zoom_speed);
    }
    if window.is_key_down(Key::N) {
      camera.zoom(-zoom_speed);
    }
}
