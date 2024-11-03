use gl::types::*;
use na::{vector, Matrix4, Rotation3, Translation3, Vector3};

#[derive(Debug)]
pub struct Camera {
    pub distance: GLfloat,
    pub default_distance: GLfloat,
    pub longitude: GLfloat,
    pub latitude: GLfloat,
    pub target: Vector3<GLfloat>,
}

impl Camera {
    pub fn new(
        default_distance: GLfloat,
        longitude: GLfloat,
        latitude: GLfloat,
        target: Vector3<GLfloat>,
    ) -> Self {
        Self {
            distance: default_distance,
            default_distance,
            longitude,
            latitude,
            target,
        }
    }

    pub fn transform(&self) -> Matrix4<GLfloat> {
        let y_axis = Vector3::y_axis();
        let x_axis: na::Unit<na::Matrix<f32, na::Const<3>, na::Const<1>, na::ArrayStorage<f32, 3, 1>>> = Vector3::x_axis();
        let latitude_rotation =
            Rotation3::from_axis_angle(&x_axis, self.latitude).to_homogeneous();
        let longitude_rotation = Rotation3::from_axis_angle(&y_axis, self.longitude).to_homogeneous();
        let relative_position_homogeneous =
            longitude_rotation * latitude_rotation * vector![0.0, 0.0, -self.distance, 1.0];
        let relative_position = vector![
            relative_position_homogeneous[0],
            relative_position_homogeneous[1],
            relative_position_homogeneous[2]
        ];
        let relative_translation = Translation3::from(relative_position);

        let translation = Translation3::from(-self.target).to_homogeneous()
            * relative_translation.to_homogeneous();
        let orientation = Rotation3::look_at_lh(&-relative_position, &y_axis).to_homogeneous();
        orientation * translation
    }

    pub fn position(&self) -> (GLfloat, GLfloat, GLfloat) {
        let y_axis = Vector3::y_axis();
        let x_axis = Vector3::x_axis();
        let longitude_rotation =
            Rotation3::from_axis_angle(&x_axis, self.latitude).to_homogeneous();
        let latitude_rotation = Rotation3::from_axis_angle(&y_axis, self.longitude).to_homogeneous();
        let relative_position_homogeneous =
            latitude_rotation * longitude_rotation * vector![0.0, 0.0, -self.distance, 1.0];
        let relative_position = vector![
            relative_position_homogeneous[0],
            relative_position_homogeneous[1],
            relative_position_homogeneous[2]
        ];

        (
            self.target.x - relative_position.x,
            self.target.y - relative_position.y,
            self.target.z - relative_position.z,
        )
    }
}
