use crate::tensor::Tensor;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq)]
pub struct Vector3 {
    tensor: Tensor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitVector3 {
    vector: Vector3,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self::from_array([x, y, z])
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn from_array(components: [f64; 3]) -> Self {
        Self {
            tensor: Tensor::vector3(components),
        }
    }

    pub fn to_array(&self) -> [f64; 3] {
        self.tensor.as_vector3()
    }

    pub fn x(&self) -> f64 {
        self.to_array()[0]
    }

    pub fn y(&self) -> f64 {
        self.to_array()[1]
    }

    pub fn z(&self) -> f64 {
        self.to_array()[2]
    }

    pub fn dot(&self, other: &Self) -> f64 {
        let [ax, ay, az] = self.to_array();
        let [bx, by, bz] = other.to_array();
        ax * bx + ay * by + az * bz
    }

    pub fn cross(&self, other: &Self) -> Self {
        let [ax, ay, az] = self.to_array();
        let [bx, by, bz] = other.to_array();
        Self::new(ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx)
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.dot(self)
    }

    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    pub fn normalized(&self) -> Option<UnitVector3> {
        UnitVector3::new(self.clone())
    }
}

impl UnitVector3 {
    pub fn new(vector: Vector3) -> Option<Self> {
        let magnitude = vector.magnitude();
        if magnitude <= f64::EPSILON {
            return None;
        }
        let normalized = vector / magnitude;
        Some(Self { vector: normalized })
    }

    pub fn as_vector(&self) -> &Vector3 {
        &self.vector
    }

    pub fn to_array(&self) -> [f64; 3] {
        self.vector.to_array()
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let [ax, ay, az] = self.to_array();
        let [bx, by, bz] = rhs.to_array();
        Self::new(ax + bx, ay + by, az + bz)
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let [ax, ay, az] = self.to_array();
        let [bx, by, bz] = rhs.to_array();
        Self::new(ax - bx, ay - by, az - bz)
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        let [x, y, z] = self.to_array();
        Self::new(x * rhs, y * rhs, z * rhs)
    }
}

impl Div<f64> for Vector3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        let [x, y, z] = self.to_array();
        Self::new(x / rhs, y / rhs, z / rhs)
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let [x, y, z] = self.to_array();
        Self::new(-x, -y, -z)
    }
}

#[cfg(test)]
mod tests {
    use super::Vector3;

    #[test]
    fn vector_dot_and_cross_work() {
        let x = Vector3::new(1.0, 0.0, 0.0);
        let y = Vector3::new(0.0, 1.0, 0.0);
        let z = x.cross(&y);
        assert_eq!(x.dot(&y), 0.0);
        assert_eq!(z.to_array(), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn vector_normalization_produces_unit_vector() {
        let vector = Vector3::new(3.0, 0.0, 4.0);
        let unit = vector.normalized().expect("vector should normalize");
        let [x, y, z] = unit.to_array();
        assert!((x - 0.6).abs() < 1e-9);
        assert!(y.abs() < 1e-9);
        assert!((z - 0.8).abs() < 1e-9);
    }
}
