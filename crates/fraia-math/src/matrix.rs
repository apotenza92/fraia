use crate::tensor::Tensor;
use crate::vector::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix3 {
    tensor: Tensor,
}

impl Matrix3 {
    pub fn new(rows: [[f64; 3]; 3]) -> Self {
        Self {
            tensor: Tensor::matrix3(rows),
        }
    }

    pub fn identity() -> Self {
        Self::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn rows(&self) -> [[f64; 3]; 3] {
        self.tensor.as_matrix3()
    }

    pub fn transpose(&self) -> Self {
        let rows = self.rows();
        Self::new([
            [rows[0][0], rows[1][0], rows[2][0]],
            [rows[0][1], rows[1][1], rows[2][1]],
            [rows[0][2], rows[1][2], rows[2][2]],
        ])
    }

    pub fn multiply_vector(&self, vector: &Vector3) -> Vector3 {
        let rows = self.rows();
        let [x, y, z] = vector.to_array();
        Vector3::new(
            rows[0][0] * x + rows[0][1] * y + rows[0][2] * z,
            rows[1][0] * x + rows[1][1] * y + rows[1][2] * z,
            rows[2][0] * x + rows[2][1] * y + rows[2][2] * z,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix3;
    use crate::Vector3;

    #[test]
    fn identity_matrix_preserves_vector() {
        let matrix = Matrix3::identity();
        let vector = Vector3::new(2.0, -1.0, 5.0);
        assert_eq!(matrix.multiply_vector(&vector).to_array(), [2.0, -1.0, 5.0]);
    }

    #[test]
    fn transpose_flips_rows_and_columns() {
        let matrix = Matrix3::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        assert_eq!(matrix.transpose().rows()[0], [1.0, 4.0, 7.0]);
    }
}
