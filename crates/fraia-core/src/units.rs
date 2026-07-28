pub use fraia_physics::*;

#[cfg(test)]
mod tests {
    use super::{
        QuantityKind, UnitParseError, format_quantity, metric_structural_unit_profile,
        parse_quantity,
    };

    #[test]
    fn formats_metric_structural_quantities() {
        let profile = metric_structural_unit_profile();
        assert_eq!(
            format_quantity(10_000.0, QuantityKind::Force, &profile),
            "10 kN"
        );
        assert_eq!(
            format_quantity(5_000.0, QuantityKind::LineLoad, &profile),
            "5 kN/m"
        );
        assert_eq!(
            format_quantity(6.0, QuantityKind::Length, &profile),
            "6000 mm"
        );
        assert_eq!(
            format_quantity(0.012, QuantityKind::Displacement, &profile),
            "12 mm"
        );
    }

    #[test]
    fn parses_supported_metric_load_units() {
        assert_eq!(
            parse_quantity("5 kN/m", QuantityKind::LineLoad).unwrap(),
            5000.0
        );
        assert_eq!(
            parse_quantity("5 kn per m", QuantityKind::LineLoad).unwrap(),
            5000.0
        );
        assert_eq!(
            parse_quantity("10 kN", QuantityKind::Force).unwrap(),
            10_000.0
        );
        assert_eq!(
            parse_quantity("5000 N/m", QuantityKind::LineLoad).unwrap(),
            5000.0
        );
        assert_eq!(
            parse_quantity("10000 N", QuantityKind::Force).unwrap(),
            10_000.0
        );
    }

    #[test]
    fn rejects_ambiguous_load_magnitude_when_kind_requires_units() {
        assert_eq!(
            parse_quantity("use 5", QuantityKind::LineLoad),
            Err(UnitParseError::MissingUnit)
        );
        assert_eq!(
            parse_quantity("use 5 kN", QuantityKind::LineLoad),
            Err(UnitParseError::WrongUnit)
        );
    }
}
