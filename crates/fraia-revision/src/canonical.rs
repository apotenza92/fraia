//! Canonical authored-model serialization.
//!
//! `fraia.structural-model.cjson.v1` is canonical JSON with UTF-8 bytes and
//! no insignificant whitespace. Object keys are lexicographically ordered;
//! model collections whose order is not engineering meaning are ordered by
//! stable object identity; semantic tags and builder object references are
//! ordered lexicographically. Ordered geometry, such as a plate boundary, is
//! deliberately not reordered. Numbers use `serde_json`'s shortest stable
//! representation, except every negative zero is rendered as `0`. This
//! format includes all authored `StructuralModel` fields and deliberately
//! excludes timestamps, UI state, conversations, and process-local state.

use crate::{CanonicalFormatVersion, CanonicalStructuralModelSerializer};
use fraia_core::StructuralModel;
use serde_json::Value;
use std::{cmp::Ordering, fmt};

/// The format version embedded in snapshot manifests and canonical bytes.
pub const STRUCTURAL_MODEL_CANONICAL_FORMAT: &str = "fraia.structural-model.cjson.v1";

#[derive(Debug)]
pub enum CanonicalizationError {
    Serialize(serde_json::Error),
}

impl fmt::Display for CanonicalizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(formatter, "could not canonicalize model: {error}"),
        }
    }
}

impl std::error::Error for CanonicalizationError {}

/// Stateless implementation of the v1 authored-model canonicalization rules.
#[derive(Debug, Clone)]
pub struct CanonicalJsonStructuralModelSerializer {
    format_version: CanonicalFormatVersion,
}

impl Default for CanonicalJsonStructuralModelSerializer {
    fn default() -> Self {
        Self {
            format_version: CanonicalFormatVersion::new(STRUCTURAL_MODEL_CANONICAL_FORMAT),
        }
    }
}

impl CanonicalJsonStructuralModelSerializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn canonicalize(&self, model: &StructuralModel) -> Result<Vec<u8>, CanonicalizationError> {
        let mut value = serde_json::to_value(model).map_err(CanonicalizationError::Serialize)?;
        normalize_structural_model(&mut value);

        let mut output = String::new();
        write_canonical_json(&value, &mut output);
        Ok(output.into_bytes())
    }
}

impl CanonicalStructuralModelSerializer for CanonicalJsonStructuralModelSerializer {
    type Error = CanonicalizationError;

    fn format_version(&self) -> &CanonicalFormatVersion {
        &self.format_version
    }

    fn serialize(&self, model: &StructuralModel) -> Result<Vec<u8>, Self::Error> {
        self.canonicalize(model)
    }
}

fn normalize_structural_model(value: &mut Value) {
    let Some(model) = value.as_object_mut() else {
        return;
    };

    for collection in [
        "nodes",
        "members",
        "plates",
        "supports",
        "loads",
        "releases",
        "load_cases",
        "builder_node_materializations",
    ] {
        if let Some(Value::Array(items)) = model.get_mut(collection) {
            match collection {
                "builder_node_materializations" => {
                    for item in items.iter_mut() {
                        if let Some(object) = item.as_object_mut() {
                            if let Some(Value::Array(refs)) = object.get_mut("object_refs") {
                                refs.sort_by(canonical_value_order);
                            }
                        }
                    }
                    items.sort_by(|left, right| {
                        field_string(left, "builder_node_id")
                            .cmp(&field_string(right, "builder_node_id"))
                    });
                }
                "members" | "plates" => {
                    for item in items.iter_mut() {
                        if let Some(object) = item.as_object_mut() {
                            if let Some(Value::Array(tags)) = object.get_mut("semantic_tags") {
                                tags.sort_by(canonical_value_order);
                            }
                        }
                    }
                    items.sort_by(|left, right| {
                        field_string(left, "id").cmp(&field_string(right, "id"))
                    });
                }
                _ => items.sort_by(|left, right| {
                    field_string(left, "id").cmp(&field_string(right, "id"))
                }),
            }
        }
    }
}

fn field_string<'a>(value: &'a Value, field: &str) -> &'a str {
    value
        .as_object()
        .and_then(|object| object.get(field))
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn canonical_value_order(left: &Value, right: &Value) -> Ordering {
    let mut left_encoded = String::new();
    let mut right_encoded = String::new();
    write_canonical_json(left, &mut left_encoded);
    write_canonical_json(right, &mut right_encoded);
    left_encoded.cmp(&right_encoded)
}

fn write_canonical_json(value: &Value, output: &mut String) {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(boolean) => output.push_str(if *boolean { "true" } else { "false" }),
        Value::Number(number) => {
            if number.as_f64() == Some(0.0) {
                output.push('0');
            } else {
                output.push_str(&number.to_string());
            }
        }
        Value::String(string) => {
            output.push_str(&serde_json::to_string(string).expect("strings serialize"))
        }
        Value::Array(values) => {
            output.push('[');
            for (index, item) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(item, output);
            }
            output.push(']');
        }
        Value::Object(object) => {
            output.push('{');
            for (index, (key, item)) in object
                .iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .enumerate()
            {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key).expect("object keys serialize"));
                output.push(':');
                write_canonical_json(item, output);
            }
            output.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CanonicalJsonStructuralModelSerializer, STRUCTURAL_MODEL_CANONICAL_FORMAT};
    use crate::{CanonicalStructuralModelSerializer, root_fixture};

    #[test]
    fn equivalent_unordered_collections_have_identical_canonical_bytes() {
        let serializer = CanonicalJsonStructuralModelSerializer::new();
        let fixture = root_fixture();
        let mut permuted = fixture.model.clone();
        permuted.nodes.reverse();
        permuted.members.reverse();
        permuted.supports.reverse();
        permuted.members[0]
            .semantic_tags
            .extend(["a".into(), "z".into()]);
        let mut equivalent = permuted.clone();
        equivalent.members[0].semantic_tags.reverse();

        assert_eq!(
            serializer.serialize(&permuted).unwrap(),
            serializer.serialize(&equivalent).unwrap()
        );
        assert_eq!(
            serializer.format_version().as_str(),
            STRUCTURAL_MODEL_CANONICAL_FORMAT
        );
    }

    #[test]
    fn canonicalization_normalizes_negative_zero() {
        let serializer = CanonicalJsonStructuralModelSerializer::new();
        let fixture = root_fixture();
        let mut negative_zero = fixture.model.clone();
        negative_zero.nodes[0].x = -0.0;

        assert_eq!(
            serializer.serialize(&fixture.model).unwrap(),
            serializer.serialize(&negative_zero).unwrap()
        );
    }
}
