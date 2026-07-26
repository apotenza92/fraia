use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitSystem {
    MetricStructural,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantityKind {
    Length,
    Force,
    LineLoad,
    Moment,
    Stress,
    Displacement,
    Area,
    SecondMomentArea,
    Mass,
    MassPerLength,
    Density,
}

pub trait QuantitySpec: Copy + Clone + fmt::Debug + PartialEq + Eq + 'static {
    const KIND: QuantityKind;
    const CANONICAL_UNIT: &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LengthKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForceKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineLoadKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MomentKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StressKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplacementKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AreaKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecondMomentAreaKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MassKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MassPerLengthKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DensityKind;

impl QuantitySpec for LengthKind {
    const KIND: QuantityKind = QuantityKind::Length;
    const CANONICAL_UNIT: &'static str = "m";
}
impl QuantitySpec for ForceKind {
    const KIND: QuantityKind = QuantityKind::Force;
    const CANONICAL_UNIT: &'static str = "N";
}
impl QuantitySpec for LineLoadKind {
    const KIND: QuantityKind = QuantityKind::LineLoad;
    const CANONICAL_UNIT: &'static str = "N/m";
}
impl QuantitySpec for MomentKind {
    const KIND: QuantityKind = QuantityKind::Moment;
    const CANONICAL_UNIT: &'static str = "N*m";
}
impl QuantitySpec for StressKind {
    const KIND: QuantityKind = QuantityKind::Stress;
    const CANONICAL_UNIT: &'static str = "Pa";
}
impl QuantitySpec for DisplacementKind {
    const KIND: QuantityKind = QuantityKind::Displacement;
    const CANONICAL_UNIT: &'static str = "m";
}
impl QuantitySpec for AreaKind {
    const KIND: QuantityKind = QuantityKind::Area;
    const CANONICAL_UNIT: &'static str = "m^2";
}
impl QuantitySpec for SecondMomentAreaKind {
    const KIND: QuantityKind = QuantityKind::SecondMomentArea;
    const CANONICAL_UNIT: &'static str = "m^4";
}
impl QuantitySpec for MassKind {
    const KIND: QuantityKind = QuantityKind::Mass;
    const CANONICAL_UNIT: &'static str = "kg";
}
impl QuantitySpec for MassPerLengthKind {
    const KIND: QuantityKind = QuantityKind::MassPerLength;
    const CANONICAL_UNIT: &'static str = "kg/m";
}
impl QuantitySpec for DensityKind {
    const KIND: QuantityKind = QuantityKind::Density;
    const CANONICAL_UNIT: &'static str = "kg/m^3";
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity<K: QuantitySpec> {
    value: f64,
    _kind: PhantomData<K>,
}

impl<K: QuantitySpec> Quantity<K> {
    pub fn canonical(value: f64) -> Self {
        Self {
            value,
            _kind: PhantomData,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn quantity_kind(&self) -> QuantityKind {
        K::KIND
    }

    pub fn canonical_unit(&self) -> &'static str {
        K::CANONICAL_UNIT
    }
}

pub type Length = Quantity<LengthKind>;
pub type Force = Quantity<ForceKind>;
pub type LineLoad = Quantity<LineLoadKind>;
pub type Moment = Quantity<MomentKind>;
pub type Stress = Quantity<StressKind>;
pub type Displacement = Quantity<DisplacementKind>;
pub type Area = Quantity<AreaKind>;
pub type SecondMomentArea = Quantity<SecondMomentAreaKind>;
pub type Mass = Quantity<MassKind>;
pub type MassPerLength = Quantity<MassPerLengthKind>;
pub type Density = Quantity<DensityKind>;

impl Length {
    pub fn from_meters(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn from_millimeters(value: f64) -> Self {
        Self::canonical(value * 0.001)
    }
    pub fn meters(&self) -> f64 {
        self.value
    }
}

impl Force {
    pub fn from_newtons(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn from_kilonewtons(value: f64) -> Self {
        Self::canonical(value * 1000.0)
    }
    pub fn newtons(&self) -> f64 {
        self.value
    }
}

impl LineLoad {
    pub fn from_newtons_per_meter(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn from_kilonewtons_per_meter(value: f64) -> Self {
        Self::canonical(value * 1000.0)
    }
    pub fn newtons_per_meter(&self) -> f64 {
        self.value
    }
}

impl Moment {
    pub fn from_newton_meters(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn from_kilonewton_meters(value: f64) -> Self {
        Self::canonical(value * 1000.0)
    }
    pub fn newton_meters(&self) -> f64 {
        self.value
    }
}

impl Stress {
    pub fn from_pascals(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn from_mpa(value: f64) -> Self {
        Self::canonical(value * 1_000_000.0)
    }
    pub fn pascals(&self) -> f64 {
        self.value
    }
}

impl Displacement {
    pub fn from_meters(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn from_millimeters(value: f64) -> Self {
        Self::canonical(value * 0.001)
    }
    pub fn meters(&self) -> f64 {
        self.value
    }
}

impl Area {
    pub fn from_square_meters(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn square_meters(&self) -> f64 {
        self.value
    }
}

impl SecondMomentArea {
    pub fn from_meters_to_fourth(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn meters_to_fourth(&self) -> f64 {
        self.value
    }
}

impl Mass {
    pub fn from_kilograms(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn kilograms(&self) -> f64 {
        self.value
    }
}

impl MassPerLength {
    pub fn from_kilograms_per_meter(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn kilograms_per_meter(&self) -> f64 {
        self.value
    }
}

impl Density {
    pub fn from_kilograms_per_cubic_meter(value: f64) -> Self {
        Self::canonical(value)
    }
    pub fn kilograms_per_cubic_meter(&self) -> f64 {
        self.value
    }
}

impl<K: QuantitySpec> Serialize for Quantity<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Quantity", 3)?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("quantityKind", &K::KIND)?;
        state.serialize_field("canonicalUnit", K::CANONICAL_UNIT)?;
        state.end()
    }
}

impl<'de, K: QuantitySpec> Deserialize<'de> for Quantity<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QuantityVisitor<K: QuantitySpec>(PhantomData<K>);

        impl<'de, K: QuantitySpec> Visitor<'de> for QuantityVisitor<K> {
            type Value = Quantity<K>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a canonical SI quantity object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut value: Option<f64> = None;
                let mut kind: Option<QuantityKind> = None;
                let mut canonical_unit: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "value" => value = Some(map.next_value()?),
                        "quantityKind" | "quantity_kind" => kind = Some(map.next_value()?),
                        "canonicalUnit" | "canonical_unit" => {
                            canonical_unit = Some(map.next_value()?)
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                let kind = kind.ok_or_else(|| de::Error::missing_field("quantityKind"))?;
                if kind != K::KIND {
                    return Err(de::Error::custom(format!(
                        "expected quantityKind {:?}, got {:?}",
                        K::KIND,
                        kind
                    )));
                }
                let canonical_unit =
                    canonical_unit.ok_or_else(|| de::Error::missing_field("canonicalUnit"))?;
                if canonical_unit != K::CANONICAL_UNIT {
                    return Err(de::Error::custom(format!(
                        "expected canonicalUnit {}, got {}",
                        K::CANONICAL_UNIT,
                        canonical_unit
                    )));
                }
                Ok(Quantity::canonical(value))
            }
        }

        deserializer.deserialize_map(QuantityVisitor::<K>(PhantomData))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantityVector3<K: QuantitySpec> {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    _kind: PhantomData<K>,
}

impl<K: QuantitySpec> QuantityVector3<K> {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
            _kind: PhantomData,
        }
    }

    pub fn quantity_kind(&self) -> QuantityKind {
        K::KIND
    }

    pub fn canonical_unit(&self) -> &'static str {
        K::CANONICAL_UNIT
    }
}

pub type LengthPoint3 = QuantityVector3<LengthKind>;
pub type ForceVector3 = QuantityVector3<ForceKind>;
pub type DisplacementVector3 = QuantityVector3<DisplacementKind>;

impl<K: QuantitySpec> Serialize for QuantityVector3<K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("QuantityVector3", 5)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("z", &self.z)?;
        state.serialize_field("quantityKind", &K::KIND)?;
        state.serialize_field("canonicalUnit", K::CANONICAL_UNIT)?;
        state.end()
    }
}

impl<'de, K: QuantitySpec> Deserialize<'de> for QuantityVector3<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VectorVisitor<K: QuantitySpec>(PhantomData<K>);

        impl<'de, K: QuantitySpec> Visitor<'de> for VectorVisitor<K> {
            type Value = QuantityVector3<K>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a canonical SI quantity vector object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut x: Option<f64> = None;
                let mut y: Option<f64> = None;
                let mut z: Option<f64> = None;
                let mut kind: Option<QuantityKind> = None;
                let mut canonical_unit: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "x" => x = Some(map.next_value()?),
                        "y" => y = Some(map.next_value()?),
                        "z" => z = Some(map.next_value()?),
                        "quantityKind" | "quantity_kind" => kind = Some(map.next_value()?),
                        "canonicalUnit" | "canonical_unit" => {
                            canonical_unit = Some(map.next_value()?)
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let kind = kind.ok_or_else(|| de::Error::missing_field("quantityKind"))?;
                if kind != K::KIND {
                    return Err(de::Error::custom(format!(
                        "expected quantityKind {:?}, got {:?}",
                        K::KIND,
                        kind
                    )));
                }
                let canonical_unit =
                    canonical_unit.ok_or_else(|| de::Error::missing_field("canonicalUnit"))?;
                if canonical_unit != K::CANONICAL_UNIT {
                    return Err(de::Error::custom(format!(
                        "expected canonicalUnit {}, got {}",
                        K::CANONICAL_UNIT,
                        canonical_unit
                    )));
                }

                Ok(QuantityVector3::new(
                    x.ok_or_else(|| de::Error::missing_field("x"))?,
                    y.ok_or_else(|| de::Error::missing_field("y"))?,
                    z.ok_or_else(|| de::Error::missing_field("z"))?,
                ))
            }
        }

        deserializer.deserialize_map(VectorVisitor::<K>(PhantomData))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitFormat {
    pub symbol: String,
    pub canonical_to_display: f64,
    pub precision: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitProfile {
    pub id: String,
    pub label: String,
    pub system: UnitSystem,
    pub length: UnitFormat,
    pub force: UnitFormat,
    pub line_load: UnitFormat,
    pub moment: UnitFormat,
    pub stress: UnitFormat,
    pub displacement: UnitFormat,
    #[serde(default = "default_area_format")]
    pub area: UnitFormat,
    #[serde(default = "default_second_moment_area_format")]
    pub second_moment_area: UnitFormat,
    #[serde(default = "default_mass_format")]
    pub mass: UnitFormat,
    #[serde(default = "default_mass_per_length_format")]
    pub mass_per_length: UnitFormat,
    #[serde(default = "default_density_format")]
    pub density: UnitFormat,
}

impl Default for UnitProfile {
    fn default() -> Self {
        metric_structural_unit_profile()
    }
}

pub fn metric_structural_unit_profile() -> UnitProfile {
    UnitProfile {
        id: "metric_structural".into(),
        label: "Metric structural".into(),
        system: UnitSystem::MetricStructural,
        length: UnitFormat {
            symbol: "mm".into(),
            canonical_to_display: 1000.0,
            precision: 0,
        },
        force: UnitFormat {
            symbol: "kN".into(),
            canonical_to_display: 0.001,
            precision: 3,
        },
        line_load: UnitFormat {
            symbol: "kN/m".into(),
            canonical_to_display: 0.001,
            precision: 3,
        },
        moment: UnitFormat {
            symbol: "kN*m".into(),
            canonical_to_display: 0.001,
            precision: 3,
        },
        stress: UnitFormat {
            symbol: "MPa".into(),
            canonical_to_display: 0.000001,
            precision: 3,
        },
        displacement: UnitFormat {
            symbol: "mm".into(),
            canonical_to_display: 1000.0,
            precision: 3,
        },
        area: default_area_format(),
        second_moment_area: default_second_moment_area_format(),
        mass: default_mass_format(),
        mass_per_length: default_mass_per_length_format(),
        density: default_density_format(),
    }
}

fn default_area_format() -> UnitFormat {
    UnitFormat {
        symbol: "m^2".into(),
        canonical_to_display: 1.0,
        precision: 6,
    }
}

fn default_second_moment_area_format() -> UnitFormat {
    UnitFormat {
        symbol: "m^4".into(),
        canonical_to_display: 1.0,
        precision: 9,
    }
}

fn default_mass_format() -> UnitFormat {
    UnitFormat {
        symbol: "kg".into(),
        canonical_to_display: 1.0,
        precision: 3,
    }
}

fn default_mass_per_length_format() -> UnitFormat {
    UnitFormat {
        symbol: "kg/m".into(),
        canonical_to_display: 1.0,
        precision: 3,
    }
}

fn default_density_format() -> UnitFormat {
    UnitFormat {
        symbol: "kg/m^3".into(),
        canonical_to_display: 1.0,
        precision: 3,
    }
}

pub fn unit_format(profile: &UnitProfile, kind: QuantityKind) -> &UnitFormat {
    match kind {
        QuantityKind::Length => &profile.length,
        QuantityKind::Force => &profile.force,
        QuantityKind::LineLoad => &profile.line_load,
        QuantityKind::Moment => &profile.moment,
        QuantityKind::Stress => &profile.stress,
        QuantityKind::Displacement => &profile.displacement,
        QuantityKind::Area => &profile.area,
        QuantityKind::SecondMomentArea => &profile.second_moment_area,
        QuantityKind::Mass => &profile.mass,
        QuantityKind::MassPerLength => &profile.mass_per_length,
        QuantityKind::Density => &profile.density,
    }
}

pub fn unit_symbol(kind: QuantityKind, profile: &UnitProfile) -> &str {
    unit_format(profile, kind).symbol.as_str()
}

pub fn format_quantity(value: f64, kind: QuantityKind, profile: &UnitProfile) -> String {
    let format = unit_format(profile, kind);
    format_display_value(value * format.canonical_to_display, format)
}

pub fn format_quantity_value<K: QuantitySpec>(
    quantity: Quantity<K>,
    profile: &UnitProfile,
) -> String {
    format_quantity(quantity.value(), K::KIND, profile)
}

pub fn format_quantity_from_unit(
    value: f64,
    kind: QuantityKind,
    input_unit: &str,
    profile: &UnitProfile,
) -> String {
    let canonical = canonical_value_from_unit(value, kind, input_unit).unwrap_or(value);
    format_quantity(canonical, kind, profile)
}

pub fn canonical_value_from_unit(value: f64, kind: QuantityKind, input_unit: &str) -> Option<f64> {
    let unit = normalize_unit(input_unit);
    let factor = match kind {
        QuantityKind::Length | QuantityKind::Displacement => match unit.as_str() {
            "m" => 1.0,
            "mm" => 0.001,
            _ => return None,
        },
        QuantityKind::Force => match unit.as_str() {
            "kn" => 1000.0,
            "n" => 1.0,
            _ => return None,
        },
        QuantityKind::LineLoad => match unit.as_str() {
            "kn/m" => 1000.0,
            "n/m" => 1.0,
            _ => return None,
        },
        QuantityKind::Moment => match unit.as_str() {
            "kn*m" | "knm" => 1000.0,
            "n*m" | "nm" => 1.0,
            _ => return None,
        },
        QuantityKind::Stress => match unit.as_str() {
            "mpa" => 1_000_000.0,
            "pa" => 1.0,
            _ => return None,
        },
        QuantityKind::Area => match unit.as_str() {
            "m^2" | "m2" => 1.0,
            "mm^2" | "mm2" => 0.000001,
            _ => return None,
        },
        QuantityKind::SecondMomentArea => match unit.as_str() {
            "m^4" | "m4" => 1.0,
            "mm^4" | "mm4" => 1e-12,
            _ => return None,
        },
        QuantityKind::Mass => match unit.as_str() {
            "kg" => 1.0,
            _ => return None,
        },
        QuantityKind::MassPerLength => match unit.as_str() {
            "kg/m" => 1.0,
            _ => return None,
        },
        QuantityKind::Density => match unit.as_str() {
            "kg/m^3" | "kg/m3" => 1.0,
            _ => return None,
        },
    };
    Some(value * factor)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitParseError {
    MissingNumber,
    MissingUnit,
    WrongUnit,
}

pub fn parse_quantity(text: &str, kind: QuantityKind) -> Result<f64, UnitParseError> {
    let value = first_number(text).ok_or(UnitParseError::MissingNumber)?;
    let normalized = normalize_text(text);
    let unit = match kind {
        QuantityKind::Force => {
            if contains_line_load_unit(&normalized) {
                return Err(UnitParseError::WrongUnit);
            }
            if contains_word_unit(&normalized, "kn") {
                "kn"
            } else if contains_word_unit(&normalized, "n") {
                "n"
            } else {
                return Err(UnitParseError::MissingUnit);
            }
        }
        QuantityKind::LineLoad => {
            if contains_kn_per_m_unit(&normalized) {
                "kn/m"
            } else if contains_n_per_m_unit(&normalized) {
                "n/m"
            } else if contains_force_unit(&normalized) {
                return Err(UnitParseError::WrongUnit);
            } else {
                return Err(UnitParseError::MissingUnit);
            }
        }
        QuantityKind::Length | QuantityKind::Displacement => {
            if contains_word_unit(&normalized, "mm") {
                "mm"
            } else if contains_word_unit(&normalized, "m")
                || normalized.contains("metre")
                || normalized.contains("meter")
            {
                "m"
            } else {
                return Err(UnitParseError::MissingUnit);
            }
        }
        QuantityKind::Moment => {
            if normalized.contains("kn m") || normalized.contains("knm") {
                "kn*m"
            } else if normalized.contains("n m") || normalized.contains("nm") {
                "n*m"
            } else {
                return Err(UnitParseError::MissingUnit);
            }
        }
        QuantityKind::Stress => {
            if contains_word_unit(&normalized, "mpa") {
                "mpa"
            } else if contains_word_unit(&normalized, "pa") {
                "pa"
            } else {
                return Err(UnitParseError::MissingUnit);
            }
        }
        QuantityKind::Area
        | QuantityKind::SecondMomentArea
        | QuantityKind::Mass
        | QuantityKind::MassPerLength
        | QuantityKind::Density => return Err(UnitParseError::MissingUnit),
    };
    canonical_value_from_unit(value, kind, unit).ok_or(UnitParseError::WrongUnit)
}

pub mod serde_f64 {
    use super::*;

    pub fn serialize_length<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Length::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_length<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, LengthKind>(deserializer)
    }

    pub fn serialize_force<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Force::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_force<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, ForceKind>(deserializer)
    }

    pub fn serialize_kilonewtons_as_force<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Force::canonical(*value * 1000.0).serialize(serializer)
    }

    pub fn deserialize_force_as_kilonewtons<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_scaled_quantity_or_legacy_number::<D, ForceKind>(deserializer, 1000.0)
    }

    pub fn serialize_line_load<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        LineLoad::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_line_load<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, LineLoadKind>(deserializer)
    }

    pub fn serialize_kilonewtons_per_meter_as_line_load<S>(
        value: &f64,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        LineLoad::canonical(*value * 1000.0).serialize(serializer)
    }

    pub fn deserialize_line_load_as_kilonewtons_per_meter<'de, D>(
        deserializer: D,
    ) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_scaled_quantity_or_legacy_number::<D, LineLoadKind>(deserializer, 1000.0)
    }

    pub fn serialize_moment<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Moment::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_moment<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, MomentKind>(deserializer)
    }

    pub fn serialize_stress<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Stress::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_stress<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, StressKind>(deserializer)
    }

    pub fn serialize_displacement<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Displacement::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_displacement<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, DisplacementKind>(deserializer)
    }

    pub fn serialize_area<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Area::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_area<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, AreaKind>(deserializer)
    }

    pub fn serialize_second_moment_area<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SecondMomentArea::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_second_moment_area<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, SecondMomentAreaKind>(deserializer)
    }

    pub fn serialize_mass_per_length<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        MassPerLength::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_mass_per_length<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, MassPerLengthKind>(deserializer)
    }

    pub fn serialize_density<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Density::canonical(*value).serialize(serializer)
    }

    pub fn deserialize_density<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_quantity_or_number::<D, DensityKind>(deserializer)
    }

    fn deserialize_quantity_or_number<'de, D, K>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
        K: QuantitySpec,
    {
        struct NumberOrQuantityVisitor<K: QuantitySpec>(PhantomData<K>);

        impl<'de, K: QuantitySpec> Visitor<'de> for NumberOrQuantityVisitor<K> {
            type Value = f64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a legacy number or canonical SI quantity object")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value as f64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value as f64)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut value: Option<f64> = None;
                let mut kind: Option<QuantityKind> = None;
                let mut canonical_unit: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "value" => value = Some(map.next_value()?),
                        "quantityKind" | "quantity_kind" => kind = Some(map.next_value()?),
                        "canonicalUnit" | "canonical_unit" => {
                            canonical_unit = Some(map.next_value()?)
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let kind = kind.ok_or_else(|| de::Error::missing_field("quantityKind"))?;
                if kind != K::KIND {
                    return Err(de::Error::custom(format!(
                        "expected quantityKind {:?}, got {:?}",
                        K::KIND,
                        kind
                    )));
                }
                let canonical_unit =
                    canonical_unit.ok_or_else(|| de::Error::missing_field("canonicalUnit"))?;
                if canonical_unit != K::CANONICAL_UNIT {
                    return Err(de::Error::custom(format!(
                        "expected canonicalUnit {}, got {}",
                        K::CANONICAL_UNIT,
                        canonical_unit
                    )));
                }
                value.ok_or_else(|| de::Error::missing_field("value"))
            }
        }

        deserializer.deserialize_any(NumberOrQuantityVisitor::<K>(PhantomData))
    }

    fn deserialize_scaled_quantity_or_legacy_number<'de, D, K>(
        deserializer: D,
        canonical_per_legacy: f64,
    ) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
        K: QuantitySpec,
    {
        struct ScaledVisitor<K: QuantitySpec> {
            canonical_per_legacy: f64,
            _kind: PhantomData<K>,
        }

        impl<'de, K: QuantitySpec> Visitor<'de> for ScaledVisitor<K> {
            type Value = f64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a legacy display-unit number or canonical SI quantity object")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value as f64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value as f64)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut value: Option<f64> = None;
                let mut kind: Option<QuantityKind> = None;
                let mut canonical_unit: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "value" => value = Some(map.next_value()?),
                        "quantityKind" | "quantity_kind" => kind = Some(map.next_value()?),
                        "canonicalUnit" | "canonical_unit" => {
                            canonical_unit = Some(map.next_value()?)
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let kind = kind.ok_or_else(|| de::Error::missing_field("quantityKind"))?;
                if kind != K::KIND {
                    return Err(de::Error::custom(format!(
                        "expected quantityKind {:?}, got {:?}",
                        K::KIND,
                        kind
                    )));
                }
                let canonical_unit =
                    canonical_unit.ok_or_else(|| de::Error::missing_field("canonicalUnit"))?;
                if canonical_unit != K::CANONICAL_UNIT {
                    return Err(de::Error::custom(format!(
                        "expected canonicalUnit {}, got {}",
                        K::CANONICAL_UNIT,
                        canonical_unit
                    )));
                }
                value
                    .map(|value| value / self.canonical_per_legacy)
                    .ok_or_else(|| de::Error::missing_field("value"))
            }
        }

        deserializer.deserialize_any(ScaledVisitor::<K> {
            canonical_per_legacy,
            _kind: PhantomData,
        })
    }
}

fn format_display_value(value: f64, format: &UnitFormat) -> String {
    let number = if value.is_finite() {
        trim_fixed(value, format.precision)
    } else {
        "n/a".into()
    };
    if format.symbol.is_empty() {
        number
    } else {
        format!("{} {}", number, format.symbol)
    }
}

fn trim_fixed(value: f64, precision: usize) -> String {
    let mut text = format!("{value:.precision$}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text == "-0" { "0".into() } else { text }
}

fn normalize_unit(unit: &str) -> String {
    unit.trim()
        .to_ascii_lowercase()
        .replace(' ', "")
        .replace("·", "*")
}

fn normalize_text(text: &str) -> String {
    text.to_ascii_lowercase()
        .replace("kilonewtons", "kn")
        .replace("kilonewton", "kn")
        .replace("newtons", "n")
        .replace("newton", "n")
        .replace("metres", "m")
        .replace("metre", "m")
        .replace("meters", "m")
        .replace("meter", "m")
        .replace(" per ", "/")
        .replace('\\', "/")
}

fn contains_force_unit(text: &str) -> bool {
    contains_word_unit(text, "kn") || contains_word_unit(text, "n")
}

fn contains_line_load_unit(text: &str) -> bool {
    contains_kn_per_m_unit(text) || contains_n_per_m_unit(text)
}

fn contains_kn_per_m_unit(text: &str) -> bool {
    text.contains("kn/m") || text.contains("kn / m") || text.contains("kn per m")
}

fn contains_n_per_m_unit(text: &str) -> bool {
    text.contains("n/m") || text.contains("n / m") || text.contains("n per m")
}

fn contains_word_unit(text: &str, unit: &str) -> bool {
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '/')
        .any(|part| part == unit)
}

fn first_number(text: &str) -> Option<f64> {
    text.split(|character: char| {
        !(character.is_ascii_digit() || character == '.' || character == '-')
    })
    .filter(|token| !token.is_empty())
    .find_map(|token| {
        token
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && *value > 0.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_scalar_quantities_as_canonical_objects() {
        let json = serde_json::to_value(LineLoad::from_newtons_per_meter(5000.0)).unwrap();
        assert_eq!(json["value"], 5000.0);
        assert_eq!(json["quantityKind"], "line_load");
        assert_eq!(json["canonicalUnit"], "N/m");
    }

    #[test]
    fn serializes_length_points_as_canonical_objects() {
        let json = serde_json::to_value(LengthPoint3::new(0.0, 6.0, 0.0)).unwrap();
        assert_eq!(json["x"], 0.0);
        assert_eq!(json["y"], 6.0);
        assert_eq!(json["z"], 0.0);
        assert_eq!(json["quantityKind"], "length");
        assert_eq!(json["canonicalUnit"], "m");
    }

    #[test]
    fn converts_supported_metric_units() {
        assert_eq!(
            canonical_value_from_unit(5.0, QuantityKind::LineLoad, "kN/m"),
            Some(5000.0)
        );
        assert_eq!(
            canonical_value_from_unit(10.0, QuantityKind::Force, "kN"),
            Some(10_000.0)
        );
        assert_eq!(
            canonical_value_from_unit(250.0, QuantityKind::Length, "mm"),
            Some(0.25)
        );
        assert_eq!(
            canonical_value_from_unit(300.0, QuantityKind::Stress, "MPa"),
            Some(300_000_000.0)
        );
    }

    #[test]
    fn rejects_mismatched_units() {
        assert_eq!(
            canonical_value_from_unit(5.0, QuantityKind::LineLoad, "kN"),
            None
        );
        assert_eq!(
            parse_quantity("use 5 kN", QuantityKind::LineLoad),
            Err(UnitParseError::WrongUnit)
        );
    }

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
}
