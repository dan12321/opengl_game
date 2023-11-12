use gl::types::*;
use na::{Vector3, Matrix4, Rotation3, Translation3, vector};

pub struct Camera {
    pub distance: GLfloat,
    pub default_distance: GLfloat,
    pub latitude: GLfloat,
    pub longitude: GLfloat,
    pub target: Vector3<GLfloat>,
}

impl Camera {
    pub fn new(default_distance: GLfloat, latitude: GLfloat, longitude: GLfloat, target: Vector3<GLfloat>) -> Self {
        Self {
            distance: default_distance,
            default_distance,
            latitude,
            longitude,
            target
        }
    }

    pub fn transform(&self) -> Matrix4<GLfloat> {
        let y_axis = Vector3::y_axis();
        let x_axis = Vector3::x_axis();
        let longitude_rotation = Rotation3::from_axis_angle(&x_axis, self.longitude).to_homogeneous();
        let latitude_rotation = Rotation3::from_axis_angle(&y_axis, self.latitude).to_homogeneous();
        let relative_position_homogeneous = latitude_rotation * longitude_rotation * vector![0.0, 0.0, -self.distance, 1.0];
        let relative_position = vector![ relative_position_homogeneous[0], relative_position_homogeneous[1], relative_position_homogeneous[2] ];
        let relative_translation = Translation3::from(relative_position);
        
        let translation = Translation3::from(-self.target).to_homogeneous() * relative_translation.to_homogeneous();
        let orientation = Rotation3::look_at_lh(&-relative_position, &y_axis).to_homogeneous();
        orientation * translation
    }
}
