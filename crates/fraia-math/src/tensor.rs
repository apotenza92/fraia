#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Tensor {
    rank: usize,
    shape: Vec<usize>,
    components: Vec<f64>,
}

impl Tensor {
    pub(crate) fn new(rank: usize, shape: Vec<usize>, components: Vec<f64>) -> Self {
        assert_eq!(
            rank,
            shape.len(),
            "tensor rank must match shape dimensionality"
        );
        let expected_components = shape.iter().product::<usize>().max(1);
        assert_eq!(
            expected_components,
            components.len(),
            "tensor component count must match shape"
        );
        Self {
            rank,
            shape,
            components,
        }
    }

    pub(crate) fn scalar(value: f64) -> Self {
        Self {
            rank: 0,
            shape: Vec::new(),
            components: vec![value],
        }
    }

    pub(crate) fn vector3(components: [f64; 3]) -> Self {
        Self::new(1, vec![3], components.to_vec())
    }

    pub(crate) fn matrix3(rows: [[f64; 3]; 3]) -> Self {
        Self::new(2, vec![3, 3], rows.into_iter().flatten().collect())
    }

    #[cfg(test)]
    pub(crate) fn rank(&self) -> usize {
        self.rank
    }

    #[cfg(test)]
    pub(crate) fn shape(&self) -> &[usize] {
        &self.shape
    }

    #[cfg(test)]
    pub(crate) fn components(&self) -> &[f64] {
        &self.components
    }

    pub(crate) fn as_scalar(&self) -> f64 {
        debug_assert_eq!(self.rank, 0);
        self.components[0]
    }

    pub(crate) fn as_vector3(&self) -> [f64; 3] {
        debug_assert_eq!(self.rank, 1);
        debug_assert_eq!(self.shape, vec![3]);
        [self.components[0], self.components[1], self.components[2]]
    }

    pub(crate) fn as_matrix3(&self) -> [[f64; 3]; 3] {
        debug_assert_eq!(self.rank, 2);
        debug_assert_eq!(self.shape, vec![3, 3]);
        [
            [self.components[0], self.components[1], self.components[2]],
            [self.components[3], self.components[4], self.components[5]],
            [self.components[6], self.components[7], self.components[8]],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::Tensor;

    #[test]
    fn scalar_tensor_has_rank_zero() {
        let tensor = Tensor::scalar(12.5);
        assert_eq!(tensor.rank(), 0);
        assert!(tensor.shape().is_empty());
        assert_eq!(tensor.components(), &[12.5]);
    }

    #[test]
    fn vector_tensor_has_rank_one() {
        let tensor = Tensor::vector3([1.0, 2.0, 3.0]);
        assert_eq!(tensor.rank(), 1);
        assert_eq!(tensor.shape(), &[3]);
        assert_eq!(tensor.as_vector3(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn matrix_tensor_has_rank_two() {
        let tensor = Tensor::matrix3([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        assert_eq!(tensor.rank(), 2);
        assert_eq!(tensor.shape(), &[3, 3]);
        assert_eq!(tensor.as_matrix3()[1][2], 6.0);
    }
}
