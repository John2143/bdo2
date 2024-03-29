use bevy::prelude::*;

#[inline]
#[allow(dead_code)]
fn remap<T>(src: T, (src_min, src_max): (T, T), (dest_min, dest_max): (T, T)) -> T
where
    T: Copy
        + std::cmp::PartialOrd
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>,
{
    if src <= src_min {
        return dest_min;
    } else if src >= src_max {
        return dest_max;
    }

    let off = src - src_min;
    let off_pct = off / (src_max - src_min);

    let dest_range = dest_max - dest_min;

    dest_min + dest_range * off_pct
}

///This trait represents a vector that can be rotated around its origin by
///some amount theta
pub trait RotatableVector<RotateInput> {
    fn rotate_ang(self, theta: RotateInput) -> Self;
}

///Rotates around origin counter clockwise
impl RotatableVector<f32> for Vec2 {
    fn rotate_ang(self, theta: f32) -> Self {
        let ang = self.x.atan2(self.y);
        let ang = (ang + theta).sin_cos();
        Vec2::new(ang.1, ang.0) * self.length()
    }
}

impl RotatableVector<(f32, f32)> for Vec3 {
    fn rotate_ang(self, (_, _): (f32, f32)) -> Self {
        todo!()
    }
}

impl RotatableVector<Quat> for Vec3 {
    fn rotate_ang(self, _quat: Quat) -> Self {
        todo!()
    }
}

pub trait Vec3toVec2 {
    fn xz2(self) -> Vec2;
}

impl Vec3toVec2 for Vec3 {
    fn xz2(self) -> Vec2 {
        Vec2::new(self.x, self.z)
    }
}

pub trait Vec2toVec3 {
    fn xz3(self) -> Vec3;
    fn xz3_withy(self, y: f32) -> Vec3;
}

impl Vec2toVec3 for Vec2 {
    fn xz3(self) -> Vec3 {
        self.xz3_withy(0.0)
    }

    fn xz3_withy(self, y: f32) -> Vec3 {
        Vec3::new(self.x, y, self.y)
    }
}
