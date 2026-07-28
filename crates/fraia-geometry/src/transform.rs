use crate::{Frame3, Point3};
use fraia_math::{Matrix3, Vector3};

#[derive(Debug, Clone, PartialEq)]
pub struct Transform3 {
    source_frame: Frame3,
    target_frame: Frame3,
    rotation: Matrix3,
    translation: Vector3,
}

impl Transform3 {
    pub fn new(
        source_frame: Frame3,
        target_frame: Frame3,
        rotation: Matrix3,
        translation: Vector3,
    ) -> Self {
        Self {
            source_frame,
            target_frame,
            rotation,
            translation,
        }
    }

    pub fn identity(frame: Frame3) -> Self {
        Self::new(frame.clone(), frame, Matrix3::identity(), Vector3::zero())
    }

    pub fn source_frame(&self) -> &Frame3 {
        &self.source_frame
    }

    pub fn target_frame(&self) -> &Frame3 {
        &self.target_frame
    }

    pub fn rotation(&self) -> &Matrix3 {
        &self.rotation
    }

    pub fn translation(&self) -> &Vector3 {
        &self.translation
    }

    pub fn apply_vector(&self, vector: &Vector3) -> Vector3 {
        self.rotation.multiply_vector(vector)
    }

    pub fn apply_point(&self, point: &Point3) -> Point3 {
        let rotated = self.rotation.multiply_vector(point.coordinates());
        Point3::from_vector(rotated + self.translation.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Transform3;
    use crate::Frame3;
    use crate::Point3;
    use fraia_math::{Matrix3, Vector3};

    #[test]
    fn identity_transform_preserves_point() {
        let transform = Transform3::identity(Frame3::global());
        let point = Point3::new(2.0, 3.0, 4.0);
        assert_eq!(transform.apply_point(&point), point);
    }

    #[test]
    fn translation_moves_point() {
        let frame = Frame3::global();
        let transform = Transform3::new(
            frame.clone(),
            frame,
            Matrix3::identity(),
            Vector3::new(1.0, -2.0, 0.5),
        );
        let point = Point3::new(2.0, 3.0, 4.0);
        let transformed = transform.apply_point(&point);
        assert_eq!(
            (transformed.x(), transformed.y(), transformed.z()),
            (3.0, 1.0, 4.5)
        );
    }
}
