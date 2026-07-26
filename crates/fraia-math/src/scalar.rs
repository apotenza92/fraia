use crate::tensor::Tensor;

#[derive(Debug, Clone, PartialEq)]
pub struct Scalar {
    tensor: Tensor,
}

impl Scalar {
    pub fn new(value: f64) -> Self {
        Self {
            tensor: Tensor::scalar(value),
        }
    }

    pub fn value(&self) -> f64 {
        self.tensor.as_scalar()
    }
}

impl From<f64> for Scalar {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Scalar;

    #[test]
    fn scalar_wraps_rank_zero_tensor() {
        let scalar = Scalar::new(8.0);
        assert_eq!(scalar.value(), 8.0);
    }
}
