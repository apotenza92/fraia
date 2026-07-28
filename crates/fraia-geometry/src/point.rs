use fraia_math::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub struct Point3 {
    coordinates: Vector3,
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            coordinates: Vector3::new(x, y, z),
        }
    }

    pub fn from_vector(coordinates: Vector3) -> Self {
        Self { coordinates }
    }

    pub fn coordinates(&self) -> &Vector3 {
        &self.coordinates
    }

    pub fn x(&self) -> f64 {
        self.coordinates.x()
    }

    pub fn y(&self) -> f64 {
        self.coordinates.y()
    }

    pub fn z(&self) -> f64 {
        self.coordinates.z()
    }

    pub fn translated(&self, offset: &Vector3) -> Self {
        Self::from_vector(self.coordinates.clone() + offset.clone())
    }

    pub fn vector_to(&self, other: &Self) -> Vector3 {
        other.coordinates.clone() - self.coordinates.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::Point3;
    use fraia_math::Vector3;

    #[test]
    fn point_translation_preserves_point_semantics() {
        let point = Point3::new(1.0, 2.0, 3.0);
        let translated = point.translated(&Vector3::new(2.0, -1.0, 0.5));
        assert_eq!(
            (translated.x(), translated.y(), translated.z()),
            (3.0, 1.0, 3.5)
        );
    }
}
