use super::state::scenes::XYZ;

pub struct Sides {
    front: f32,
    back: f32,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

pub const SIDES: Sides = Sides {
    front: 0.5,
    back: -0.5,
    left: -0.5,
    right: 0.5,
    top: 0.5,
    bottom: -0.5,
};

#[derive(Debug)]
pub struct AABBColider {
    pub position: XYZ,
    pub scale: XYZ,
}

impl AABBColider {
    pub fn aabb_colided(&self, cube: &AABBColider) -> bool {
        let a = self.to_sides();
        let b = cube.to_sides();
        a.back <= b.front
            && a.front >= b.back
            && a.left <= b.right
            && a.right >= b.left
            && a.bottom <= b.top
            && a.top >= b.bottom
    }

    fn to_sides(&self) -> Sides {
        Sides {
            front: SIDES.front * self.scale.x + self.position.x,
            back: SIDES.back * self.scale.x + self.position.x,
            left: SIDES.left * self.scale.y + self.position.y,
            right: SIDES.right * self.scale.y + self.position.y,
            top: SIDES.top * self.scale.z + self.position.z,
            bottom: SIDES.bottom * self.scale.z + self.position.z,
        }
    }
}
