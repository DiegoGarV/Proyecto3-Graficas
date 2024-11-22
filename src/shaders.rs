use nalgebra_glm::{Vec2, Vec3, Vec4, Mat3, dot, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragments::Fragments;
use crate::color::Color;
use std::f32::consts::PI;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  let position = Vec4::new(
    vertex.position.x,
    vertex.position.y,
    vertex.position.z,
    1.0
  );
  let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

  let w = transformed.w;
  let ndc_position = Vec4::new(
    transformed.x / w,
    transformed.y / w,
    transformed.z / w,
    1.0
  );

  let screen_position = uniforms.viewport_matrix * ndc_position;

  let model_mat3 = mat4_to_mat3(&uniforms.model_matrix); 
  let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

  let transformed_normal = normal_matrix * vertex.normal;

  Vertex {
    position: vertex.position,
    normal: vertex.normal,
    tex_coords: vertex.tex_coords,
    color: vertex.color,
    transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
    transformed_normal,
  }
}

pub enum ShaderType {
  Sun,
  Earth,
  GasPlanet,
  RingPlanet,
  RockyPlanet,
  IcyPlanet,
  VolcanicPlanet,
  Moon,
  Ring,
}

pub fn fragment_shader(fragment: &Fragments, uniforms: &Uniforms, current_shader: &ShaderType) -> Color {
  match current_shader {
    ShaderType::Sun => sun_shader(fragment, uniforms),
    ShaderType::Earth => earth_shader(fragment, uniforms),
    ShaderType::GasPlanet => gas_planet_shader(fragment, uniforms),
    ShaderType::RingPlanet => ring_planet_shader(fragment, uniforms),
    ShaderType::RockyPlanet => rocky_planet_shader(fragment, uniforms),
    ShaderType::IcyPlanet => icy_planet_shader(fragment, uniforms),
    ShaderType::VolcanicPlanet => volcanic_planet_shader(fragment, uniforms),
    ShaderType::Moon => moon_shader(fragment, uniforms),
    ShaderType::Ring => ring_shader(fragment, uniforms),
  }
}



// Planeta de hielo
pub fn icy_planet_shader(fragment: &Fragments, uniforms: &Uniforms) -> Color {
  let base_color = Color::new(173, 216, 230); // Celeste
  let fracture_color = Color::new(255, 255, 255); // Blanco

  // Grietas
  let stripe_width = 0.15;
  let combined_pos = fragment.vertex_pos.x * 0.7 + fragment.vertex_pos.y * 0.3;
  let stripe_factor = ((combined_pos / stripe_width) * PI).sin().abs();

  let fracture_factor = (1.0 - stripe_factor).powf(3.0);
  let fractured_surface = base_color.lerp(&fracture_color, fracture_factor);

  // Reflejo
  let normal = fragment.normal.normalize();
  let light_dir = Vec3::new(0.0, 0.0, -1.0);
  let view_dir = -fragment.vertex_pos.normalize();
  let reflect_dir = (2.0 * dot(&light_dir, &normal) * normal - light_dir).normalize();
  let specular_intensity = dot(&reflect_dir, &view_dir).max(0.0).powf(32.0);
  let specular_color = Color::new(255, 255, 255);
  let reflected_surface = fractured_surface.lerp(&specular_color, specular_intensity * 0.5);

  // Depuración
  match uniforms.debug_mode {
      1 => base_color * fragment.intensity,            // Solo el color base
      2 => fracture_color * fracture_factor,           // Solo las grietas
      3 => specular_color * specular_intensity,        // Solo la reflexión especular
      _ => reflected_surface * fragment.intensity,     // Shader completo
  }
}

// Planeta volcánico
pub fn volcanic_planet_shader(fragment: &Fragments, uniforms: &Uniforms) -> Color {
  let rock_color = Color::new(50, 50, 50);    // Gris oscuro
  let lava_color = Color::new(255, 100, 0);    // Naranja más intenso (más saturado)

  // Lava
  let lava_scale = 15.0;
  let noise_x = fragment.vertex_pos.x * lava_scale + uniforms.time as f32 * 0.1;
  let noise_y = fragment.vertex_pos.y * lava_scale - uniforms.time as f32 * 0.1;
  let lava_noise = ((noise_x.sin() * noise_y.cos()).abs() * 1.5).fract();
  let lava_factor = (lava_noise - 0.7).max(0.0) / 0.3;
  let surface_color = rock_color.lerp(&lava_color, lava_factor);

  // Brillo
  let glow_factor = (lava_factor.powf(2.0) * 0.8).clamp(0.0, 1.0);
  let glow_color = lava_color.lerp(&Color::new(255, 255, 50), glow_factor);
  let final_color = surface_color.lerp(&glow_color, glow_factor);

  // Luz de la lava
  let lava_emission_factor = 0.8;
  let lava_emitted_color = lava_color * lava_emission_factor;
  let emitted_color = final_color.lerp(&lava_emitted_color, lava_factor);

  // Depuración
  match uniforms.debug_mode {
      1 => rock_color * fragment.intensity,             // Only rock color
      2 => lava_color * lava_factor,                    // Only lava regions
      3 => glow_color * glow_factor,                    // Only glow effect
      _ => emitted_color * fragment.intensity,          // Full shader with emission effect
  }
}

// Sol
pub fn sun_shader(fragment: &Fragments, uniforms: &Uniforms) -> Color {
  // Colores base del degradado
  let color1 = Color::new(255, 255, 255); // Amarillo muy claro
  let color2 = Color::new(255, 230, 28); // Amarillo pastel
  let color3 = Color::new(255, 178, 51); // Amarillo intenso
  let color4 = Color::new(204, 102, 0);  // Naranja oscuro

  // Coordenadas del fragmento normalizadas al rango [-1, 1]
  let x = fragment.vertex_pos.x;
  let y = fragment.vertex_pos.y;

  // Centro del degradado
  let center = (0.0, 0.0);
  let radius = ((x - center.0).powi(2) + (y - center.1).powi(2)).sqrt();

  // Radio normalizado entre 0 y 1
  let t = radius.clamp(0.0, 1.0);

  // Mezcla de colores según el radio
  let blended_color = if t < 0.33 {
      color1.lerp(&color2, t / 0.33)
  } else if t < 0.66 {
      color2.lerp(&color3, (t - 0.33) / 0.33)
  } else {
      color3.lerp(&color4, (t - 0.66) / 0.34)
  };

  // Emisión del sol
  let emission_factor = 1.5;
  let emitted_color = blended_color * emission_factor;

  // Depuración
  match uniforms.debug_mode {
      1 => blended_color * fragment.intensity,                      // Degradado sin emisión
      2 => blended_color,                                           // Degradado puro
      3 => Color::new(255, 255, 255) * emission_factor,     // Solo emisión blanca
      _ => emitted_color * fragment.intensity,                      // Shader completo
  }
}

// Planeta gaseoso
pub fn gas_planet_shader(fragment: &Fragments, uniforms: &Uniforms) -> Color {
  let band_color1 = Color::new(139, 69, 19);  // Marrón más oscuro
  let band_color2 = Color::new(205, 133, 63); // Marrón claro
  let band_color3 = Color::new(222, 184, 135); // Beige


  // Franjas horizontales
  let band_scale = 4.0;
  let flow_speed = 0.001;
  let flow_offset = uniforms.time as f32 * flow_speed;
  let y_position = fragment.vertex_pos.y + flow_offset;
  let band_factor = ((y_position * band_scale).sin() * 0.5 + 0.5).fract();

  // Mezcla entre colores según la posición en las bandas
  let band_color = if band_factor < 0.33 {
      band_color1.lerp(&band_color2, band_factor / 0.33)
  } else if band_factor < 0.66 {
      band_color2.lerp(&band_color3, (band_factor - 0.33) / 0.33)
  } else {
      band_color3.lerp(&band_color1, (band_factor - 0.66) / 0.34)
  };

  // Vortice
  let vortex_center = Vec2::new(-0.2, -0.2);
  let vortex_radius = 0.3;
  let distance_to_vortex = ((fragment.vertex_pos.x - vortex_center.x).powi(2)
      + (fragment.vertex_pos.y - vortex_center.y).powi(2))
      .sqrt();
  let vortex_intensity = ((vortex_radius - distance_to_vortex).max(0.0f32) / vortex_radius).powf(2.0);
  let vortex_color = Color::new(255, 69, 0);
  let final_color = band_color.lerp(&vortex_color, vortex_intensity);

  // Depuración
  match uniforms.debug_mode {
      1 => band_color * fragment.intensity,       // Solo franjas
      2 => vortex_color * vortex_intensity,       // Solo vórtice
      _ => final_color * fragment.intensity,      // Shader completo
  }
}

// Planeta rocoso
pub fn rocky_planet_shader(fragment: &Fragments, _uniforms: &Uniforms) -> Color {
  // Colores base para la superficie rocosa
  let base_color = Color::new(139, 69, 19);    // Marrón rojizo oscuro
  let mid_color = Color::new(205, 92, 92);     // Rojo rosado
  let highlight_color = Color::new(255, 160, 122); // Salmón claro

  // Generar ruido para simular textura rocosa
  let rock_scale = 10.0; // Mayor escala para patrones más finos
  let detail_scale = 0.3; // Escala para detalles pequeños

  // Coordenadas ajustadas con pseudoaleatoriedad
  let x = fragment.vertex_pos.x;
  let y = fragment.vertex_pos.y;
  let randomness = (x * 12.9898 + y * 78.233).sin() * 43758.5453;
  let random_factor = randomness.fract() * detail_scale;

  // Patrón principal con variaciones añadidas
  let noise = (((x + random_factor) * rock_scale).sin() * ((y + random_factor) * rock_scale).cos()).abs();

  // Interpolación entre colores según el ruido
  let rocky_surface = if noise < 0.4 {
      base_color.lerp(&mid_color, noise / 0.4)
  } else {
      mid_color.lerp(&highlight_color, (noise - 0.4) / 0.6)
  };

  // Depuración
  rocky_surface * fragment.intensity
}

// Luna (del planeta rocoso)
pub fn moon_shader(fragment: &Fragments, _uniforms: &Uniforms) -> Color {
  // Colores base para la luna
  let base_color = Color::new(169, 169, 169);    // Gris
  let mid_color = Color::new(190, 190, 190);     // Gris medio
  let highlight_color = Color::new(211, 211, 211); // Gris claro

  // Generar ruido para simular textura rocosa
  let rock_scale = 12.0; // Escala mayor para patrones más finos
  let detail_scale = 0.25; // Escala para detalles adicionales

  // Coordenadas ajustadas con pseudoaleatoriedad
  let x = fragment.vertex_pos.x;
  let y = fragment.vertex_pos.y;
  let randomness = (x * 15.789 + y * 41.233).sin() * 43758.5453;
  let random_factor = randomness.fract() * detail_scale;

  // Patrón principal de ruido
  let noise = (((x + random_factor) * rock_scale).sin() * ((y + random_factor) * rock_scale).cos()).abs();

  // Interpolar entre colores según el ruido
  let rocky_surface = if noise < 0.5 {
      base_color.lerp(&mid_color, noise / 0.5)
  } else {
      mid_color.lerp(&highlight_color, (noise - 0.5) / 0.5)
  };

  // Configuración de cráteres
  let crater_positions = [
      (0.1, 0.2, 0.50), 
      (-0.3, -0.1, 0.30),
      (0.4, -0.3, 0.2), 
      (-0.1, 0.5, 0.40),
      (-0.5, -0.4, 0.25),
      (0.3, 0.4, 0.35),
      (0.1, 0.5, 0.20),
      (0.2, -0.1, 0.25),
      (0.0, -0.6, 0.28), 
      (-0.4, 0.2, 0.22),
      (0.5, 0.0, 0.30),  
      (-0.2, -0.5, 0.18), 
      (0.35, 0.5, 0.24),
      (-0.45, -0.3, 0.20),
  ];

  let crater_color = Color::new(100, 100, 100); // Gris oscuro para los cráteres

  // Combinar intensidades de todos los cráteres
  let mut combined_crater_intensity = 0.0;
  for &(cx, cy, radius) in crater_positions.iter() {
      let distance = ((fragment.vertex_pos.x - cx).powi(2)
          + (fragment.vertex_pos.y - cy).powi(2))
          .sqrt();
      let crater_intensity = ((radius - distance).max(0.0f32) / radius).powf(3.0);
      combined_crater_intensity += crater_intensity;
  }

  // Aplicar la intensidad de los cráteres a la superficie
  let final_surface = rocky_surface.lerp(&crater_color, combined_crater_intensity);

  // Multiplicar por la intensidad para iluminación
  final_surface * fragment.intensity
}

// Movimiento orbital de la luna
pub fn moon_position(time: f32, radius: f32) -> Vec3 {
  let angle = time * 0.01;
  Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin())
}

// planeta con anillos
pub fn ring_planet_shader(fragment: &Fragments, uniforms: &Uniforms) -> Color {
  let band_color1 = Color::new(189, 155, 107); // Marrón claro
  let band_color2 = Color::new(210, 180, 140); // Beige
  let band_color3 = Color::new(255, 222, 173); // Crema

  // Franjas horizontales
  let band_scale = 3.5; // Ajusta el número de franjas
  let flow_speed = 0.0008; // Movimiento más lento que Júpiter
  let flow_offset = uniforms.time as f32 * flow_speed;
  let y_position = fragment.vertex_pos.y + flow_offset;
  let band_factor = ((y_position * band_scale).sin() * 0.5 + 0.5).fract();

  // Mezcla entre colores según la posición en las bandas
  let band_color = if band_factor < 0.33 {
      band_color1.lerp(&band_color2, band_factor / 0.33)
  } else if band_factor < 0.66 {
      band_color2.lerp(&band_color3, (band_factor - 0.33) / 0.33)
  } else {
      band_color3.lerp(&band_color1, (band_factor - 0.66) / 0.34)
  };

  // Depuración
  match uniforms.debug_mode {
      1 => band_color * fragment.intensity, // Solo las franjas
      _ => band_color * fragment.intensity, // Shader completo
  }
}

// Anillos
fn ring_shader(fragment: &Fragments, uniforms: &Uniforms) -> Color {
  // Colores base para el anillo
  let base_color = Color::new(255, 220, 80); // Amarillo
  let shadow_color = Color::new(150, 120, 60); // Sombra

  // Interpolación de colores
  let surface_color = base_color;

  // Iluminación básica para simular sombras
  let light_direction = Vec3::new(1.0, 1.0, 1.0).normalize(); // Dirección de la luz
  let normal = fragment.vertex_pos.normalize(); // Normal del fragmento
  let light_intensity = (normal.dot(&light_direction)).clamp(0.2, 1.0); // Intensidad de la luz

  // Lógica de depuración
  let final_color = match uniforms.debug_mode {
      1 => base_color * fragment.intensity,                                                 // Solo el color base
      _ => surface_color * light_intensity + shadow_color * (1.0 - light_intensity),      // Shader completo
  };

  final_color
}

// Planeta Tierra
pub fn earth_shader(fragment: &Fragments, uniforms: &Uniforms) -> Color {
  let x = fragment.vertex_pos.x;
  let y = fragment.vertex_pos.y;
  let z = fragment.vertex_pos.z;

  // Coordenadas esféricas
  let theta = (y / 0.5).asin(); // Latitud
  let phi = z.atan2(x);         // Longitud
  let u = (phi / (2.0 * PI)) + 0.5; // Coordenada u [0, 1]
  let v = (theta / PI) + 0.5;      // Coordenada v [0, 1]

  let scale = 7.2;
  let noise = ((u * scale).sin() * (v * scale).cos()).abs();
  let continent_threshold = 0.55;

  let land_color = Color::new(34, 139, 34); // Verde para los continentes
  let ocean_color = Color::new(0, 105, 148); // Azul para el océano
  let base_color = if noise > continent_threshold { land_color } else { ocean_color };

  // Parámetros de las nubes
  let time = uniforms.time as f32 * 0.01; // Escala temporal para el movimiento de las nubes
  let cloud_scale = 8.0;                 // Escala de dispersión de las nubes
  let cloud_intensity = ((u * cloud_scale + time).sin() * (v * cloud_scale + time).cos()).abs();
  let cloud_intensity = (cloud_intensity - 0.5).clamp(0.0, 1.0) * 0.5; // Intensidad y opacidad de las nubes

  let cloud_color = Color::new(255, 255, 255); // Blanco para las nubes

  // Crear círculos de nubes (en movimiento)
  let cloud_radius = 1.5; // Radio máximo de la atmósfera con nubes
  let distance_from_center = Vec2::new(u, v).norm(); // Distancia del centro para determinar si está dentro de la atmósfera
  let is_in_atmosphere = distance_from_center < cloud_radius;

  // Reducir el número de círculos de nubes y hacerlos más pequeños
  let num_clouds = 6; // Menor número de círculos de nubes
  let mut cloud_positions = Vec::new();

  for i in 0..num_clouds {
      let angle = (i as f32 / num_clouds as f32) * 2.0 * PI + time * 0.2; // Movimiento en el tiempo
      let radius = 0.2 + (i as f32 * 0.05); // Radios de los círculos
      let x_pos = (angle.cos() * radius + 0.5) % 1.0; // Posición en u
      let y_pos = (angle.sin() * radius + 0.5) % 1.0; // Posición en v
      cloud_positions.push(Vec2::new(x_pos, y_pos));
  }

  // Dibujar las nubes en círculos
  let mut cloud_color_final = Color::new(0, 0, 0); // Comienza con un color negro
  for cloud_pos in cloud_positions.iter() {
      let frag_position = Vec2::new(u, v);
      let distance_to_cloud = (frag_position - *cloud_pos).norm(); // Distancia a cada círculo de nube
      let cloud_radius = 0.075; // Radio más pequeño para los círculos de nubes
      let is_in_cloud = distance_to_cloud < cloud_radius;

      // Si el fragmento está dentro de un círculo de nube, añade su color
      if is_in_cloud {
          cloud_color_final = cloud_color_final.lerp(&cloud_color, 0.7); // Aumentamos la mezcla para que sea más blanco
      }
  }

  // Determinar el color final
  let final_color = if is_in_atmosphere {
      // Mezclar nubes y superficie
      base_color * (1.0 - cloud_intensity) + cloud_color_final
  } else {
      base_color
  };

  final_color
}











