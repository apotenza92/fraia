use crate::Point3;
use fraia_math::{UnitVector3, Vector3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handedness {
    Right,
    Left,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame3 {
    origin: Point3,
    x_axis: UnitVector3,
    y_axis: UnitVector3,
    z_axis: UnitVector3,
    handedness: Handedness,
}

impl Frame3 {
    pub fn new(
        origin: Point3,
        x_axis: UnitVector3,
        y_axis: UnitVector3,
        z_axis: UnitVector3,
        handedness: Handedness,
    ) -> Option<Self> {
        let x = x_axis.as_vector();
        let y = y_axis.as_vector();
        let z = z_axis.as_vector();
        let orthogonality_tolerance = 1e-9;

        if x.dot(y).abs() > orthogonality_tolerance
            || y.dot(z).abs() > orthogonality_tolerance
            || x.dot(z).abs() > orthogonality_tolerance
        {
            return None;
        }

        let orientation = x.cross(y).dot(z);
        match handedness {
            Handedness::Right if orientation <= 0.0 => return None,
            Handedness::Left if orientation >= 0.0 => return None,
            _ => {}
        }

        Some(Self {
            origin,
            x_axis,
            y_axis,
            z_axis,
            handedness,
        })
    }

    pub fn global() -> Self {
        Self::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0).normalized().expect("unit x"),
            Vector3::new(0.0, 1.0, 0.0).normalized().expect("unit y"),
            Vector3::new(0.0, 0.0, 1.0).normalized().expect("unit z"),
            Handedness::Right,
        )
        .expect("global frame should be valid")
    }

    pub fn origin(&self) -> &Point3 {
        &self.origin
    }

    pub fn x_axis(&self) -> &UnitVector3 {
        &self.x_axis
    }

    pub fn y_axis(&self) -> &UnitVector3 {
        &self.y_axis
    }

    pub fn z_axis(&self) -> &UnitVector3 {
        &self.z_axis
    }

    pub fn handedness(&self) -> Handedness {
        self.handedness
    }
}

#[cfg(test)]
mod tests {
    use super::{Frame3, Handedness};
    use crate::Point3;
    use fraia_math::Vector3;

    #[test]
    fn global_frame_is_valid() {
        let frame = Frame3::global();
        assert_eq!(frame.handedness(), Handedness::Right);
        assert_eq!(frame.origin().x(), 0.0);
    }

    #[test]
    fn invalid_axes_are_rejected() {
        let x = Vector3::new(1.0, 0.0, 0.0).normalized().unwrap();
        let y = Vector3::new(1.0, 0.0, 0.0).normalized().unwrap();
        let z = Vector3::new(0.0, 0.0, 1.0).normalized().unwrap();
        assert!(Frame3::new(Point3::new(0.0, 0.0, 0.0), x, y, z, Handedness::Right).is_none());
    }
}
