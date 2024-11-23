use nalgebra_glm::{Vec3, rotate_vec3};

pub struct Camera {
  pub eye: Vec3,
  pub center: Vec3,
  pub up: Vec3,
  pub has_changed: bool
}

impl Camera {
  pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
    Camera {
      eye,
      center,
      up,
      has_changed: true,
    }
  }

  pub fn move_center(&mut self, direction: Vec3) {
    let radius_vector = self.center - self.eye;
    let radius = radius_vector.magnitude();

    let angle_x = direction.x * 0.05; // Adjust this factor to control rotation speed
    let angle_y = direction.y * 0.05;

    let rotated = rotate_vec3(&radius_vector, angle_x, &Vec3::new(0.0, 1.0, 0.0));

    let right = rotated.cross(&self.up).normalize();
    let final_rotated = rotate_vec3(&rotated, angle_y, &right);

    self.center = self.eye + final_rotated.normalize() * radius;
    self.has_changed = true;
  }

  pub fn move_ship(&mut self, direction: Vec3) {
    let forward = (self.center - self.eye).normalize(); // Dirección en la que la cámara está mirando
    let right = forward.cross(&self.up).normalize(); // Dirección lateral (perpendicular a 'forward' y 'up')
    let up = right.cross(&forward).normalize(); // Dirección hacia arriba

    // Mover la cámara en función de la entrada
    let movement = right * direction.x + up * direction.y + forward * direction.z;
    self.eye += movement;
    self.center += movement;

    self.has_changed = true;
  }
}
