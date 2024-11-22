use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;

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

fn render_rings(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    let ring_uniforms = Uniforms {
        model_matrix: create_model_matrix(Vec3::new(0.0, 0.0, 0.0), 0.6, Vec3::new(0.0, 0.0, 0.0)),
        view_matrix: uniforms.view_matrix,
        projection_matrix: uniforms.projection_matrix,
        viewport_matrix: uniforms.viewport_matrix,
        time: uniforms.time,
        debug_mode: uniforms.debug_mode,
    };
    let ring_shader = ShaderType::Ring; // Define un ShaderType para los anillos
    render(framebuffer, &ring_uniforms, vertex_array, &ring_shader);
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

fn render_scene5(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    // agrega la luna
    let moon_position = moon_position(uniforms.time as f32, 1.3);
    let moon_shader = ShaderType::Moon;

    // Llamamos a render para Marte (rocoso)
    let current_shader = ShaderType::RockyPlanet;
    render(framebuffer, uniforms, vertex_array, &current_shader);

    // Llamamos a render para la luna
    let moon_uniforms = Uniforms {
        model_matrix: create_model_matrix(moon_position, 0.5, Vec3::new(0.0, 0.0, 0.0)),
        view_matrix: uniforms.view_matrix,
        projection_matrix: uniforms.projection_matrix,
        viewport_matrix: uniforms.viewport_matrix,
        time: uniforms.time,
        debug_mode: uniforms.debug_mode,
    };
    render(framebuffer, &moon_uniforms, vertex_array, &moon_shader);
}

fn setup_scene(scene_number: u32) -> (Vec3, f32, Vec3, Vec3, Vec3) {
    match scene_number {
        1 => {
            // Escena 1: Sol
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
        2 => {
            // Escena 2: Tierra
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
        3 => {
            // Escena 3: Planeta gaseoso
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
        4 => {
            // Escena 4: Planeta con anillos
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
        5 => {
            // Escena 5: Planeta rocoso con luna
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
        6 => {
            // Escena 6: Planeta de hielo
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
        7 => {
            // Escena 7: Planeta volcanico
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
        _ => {
            // Escena predeterminada
            (Vec3::new(0.0, 0.0, 0.0), 1.0, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 1.0, 0.0))
        },
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

    let mut scene_number = 1;

    // camera parameters
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    let sphere_loader = Obj::load("models/sphere.obj").expect("Failed to load sphere obj");
    let sphere_vertex_arrays = sphere_loader.get_vertex_array();
    
    let ring_loader = Obj::load("models/ring.obj").expect("Failed to load ring obj");
    let ring_vertex_array = ring_loader.get_vertex_array();

    let mut time = 0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        // Cambiar escena
        if window.is_key_down(Key::Key1) {
            scene_number = 1;
        } else if window.is_key_down(Key::Key2) {
            scene_number = 2;
        } else if window.is_key_down(Key::Key3) {
            scene_number = 3;
        } else if window.is_key_down(Key::Key4) {
            scene_number = 4;
        } else if window.is_key_down(Key::Key5) {
            scene_number = 5;
        } else if window.is_key_down(Key::Key6) {
            scene_number = 6;
        } else if window.is_key_down(Key::Key7) {
            scene_number = 7;
        }

        let (translation, scale, rotation, _eye, _up) = setup_scene(scene_number);

        let current_shader: ShaderType;
        current_shader = match scene_number {
            1 => ShaderType::Sun,
            2 => ShaderType::Earth,
            3 => ShaderType::GasPlanet,
            4 => ShaderType::RingPlanet,
            5 => ShaderType::RockyPlanet,
            6 => ShaderType::IcyPlanet,
            7 => ShaderType::VolcanicPlanet,
            _ => ShaderType::Sun,
        };


        time += 1;

        handle_input(&window, &mut camera);

        framebuffer.clear();

        let model_matrix = create_model_matrix(translation, scale, rotation);
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);
        let debug_mode = 0;
        let uniforms = Uniforms { 
            model_matrix, 
            view_matrix, 
            projection_matrix, 
            viewport_matrix, 
            time, 
            debug_mode,
        };

        framebuffer.set_current_color(0xFFDDDD);
        render(&mut framebuffer, &uniforms, &sphere_vertex_arrays, &current_shader);

        if scene_number == 4 {
            render(&mut framebuffer, &uniforms, &sphere_vertex_arrays, &current_shader);
            render_rings(&mut framebuffer, &uniforms, &ring_vertex_array);
        }

        if scene_number == 5 {
            render_scene5(&mut framebuffer, &uniforms, &sphere_vertex_arrays);
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
