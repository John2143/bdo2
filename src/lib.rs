use bevy::prelude::*;

#[inline]
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
    fn rotate(self, theta: RotateInput) -> Self;
}

///Rotates around origin counter clockwise
impl RotatableVector<f32> for Vec2 {
    fn rotate(self, theta: f32) -> Self {
        let (x, y) = (self.x(), self.y());

        let ang = x.atan2(y);
        let ang = (ang + theta).sin_cos();
        Vec2::new(ang.1, ang.0) * self.length()
    }
}

impl RotatableVector<(f32, f32)> for Vec3 {
    fn rotate(self, (_, _): (f32, f32)) -> Self {
        todo!()
    }
}

impl RotatableVector<Quat> for Vec3 {
    fn rotate(self, _quat: Quat) -> Self {
        todo!()
    }
}
