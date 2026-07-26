use crate::types::{Material, Section};

pub fn steel_material() -> Material {
    Material {
        id: "steel.demo.s300".into(),
        name: "Demo Structural Steel".into(),
        e: 200e9,
        fy: 300e6,
        density: 7850.0,
        cost_per_kg: 3.5,
        carbon_per_kg: 1.9,
    }
}

pub fn section_catalog() -> Vec<Section> {
    vec![
        Section {
            id: "200UB".into(),
            name: "200UB".into(),
            area: 0.0030,
            i: 2.4e-5,
            depth: 0.20,
            mass_kg_per_m: 23.6,
        },
        Section {
            id: "250UB".into(),
            name: "250UB".into(),
            area: 0.0040,
            i: 5.0e-5,
            depth: 0.25,
            mass_kg_per_m: 31.4,
        },
        Section {
            id: "310UB".into(),
            name: "310UB".into(),
            area: 0.0055,
            i: 1.05e-4,
            depth: 0.31,
            mass_kg_per_m: 43.2,
        },
        Section {
            id: "360UB".into(),
            name: "360UB".into(),
            area: 0.0067,
            i: 1.70e-4,
            depth: 0.36,
            mass_kg_per_m: 52.6,
        },
        Section {
            id: "410UB".into(),
            name: "410UB".into(),
            area: 0.0076,
            i: 2.60e-4,
            depth: 0.41,
            mass_kg_per_m: 59.7,
        },
        Section {
            id: "460UB".into(),
            name: "460UB".into(),
            area: 0.0090,
            i: 3.70e-4,
            depth: 0.46,
            mass_kg_per_m: 70.7,
        },
        Section {
            id: "530UB".into(),
            name: "530UB".into(),
            area: 0.0110,
            i: 5.50e-4,
            depth: 0.53,
            mass_kg_per_m: 86.0,
        },
        Section {
            id: "610UB".into(),
            name: "610UB".into(),
            area: 0.0140,
            i: 8.50e-4,
            depth: 0.61,
            mass_kg_per_m: 110.0,
        },
        Section {
            id: "150PFC".into(),
            name: "150PFC".into(),
            area: 0.0022,
            i: 1.35e-5,
            depth: 0.15,
            mass_kg_per_m: 17.7,
        },
        Section {
            id: "200PFC".into(),
            name: "200PFC".into(),
            area: 0.0030,
            i: 2.65e-5,
            depth: 0.20,
            mass_kg_per_m: 22.9,
        },
        Section {
            id: "150UC".into(),
            name: "150UC".into(),
            area: 0.0040,
            i: 1.85e-5,
            depth: 0.15,
            mass_kg_per_m: 30.0,
        },
        Section {
            id: "200UC".into(),
            name: "200UC".into(),
            area: 0.0060,
            i: 4.80e-5,
            depth: 0.20,
            mass_kg_per_m: 46.2,
        },
        Section {
            id: "100x50RHS".into(),
            name: "100x50RHS".into(),
            area: 0.0016,
            i: 3.80e-6,
            depth: 0.10,
            mass_kg_per_m: 12.5,
        },
        Section {
            id: "150x100RHS".into(),
            name: "150x100RHS".into(),
            area: 0.0029,
            i: 1.45e-5,
            depth: 0.15,
            mass_kg_per_m: 22.8,
        },
        Section {
            id: "100SHS".into(),
            name: "100SHS".into(),
            area: 0.0019,
            i: 5.60e-6,
            depth: 0.10,
            mass_kg_per_m: 14.9,
        },
        Section {
            id: "150SHS".into(),
            name: "150SHS".into(),
            area: 0.0030,
            i: 1.65e-5,
            depth: 0.15,
            mass_kg_per_m: 23.5,
        },
        Section {
            id: "89CHS".into(),
            name: "89CHS".into(),
            area: 0.0013,
            i: 2.60e-6,
            depth: 0.089,
            mass_kg_per_m: 10.1,
        },
        Section {
            id: "114CHS".into(),
            name: "114CHS".into(),
            area: 0.0019,
            i: 6.70e-6,
            depth: 0.114,
            mass_kg_per_m: 14.9,
        },
        Section {
            id: "75EA".into(),
            name: "75EA".into(),
            area: 0.0011,
            i: 1.20e-6,
            depth: 0.075,
            mass_kg_per_m: 8.7,
        },
        Section {
            id: "100EA".into(),
            name: "100EA".into(),
            area: 0.0017,
            i: 3.20e-6,
            depth: 0.10,
            mass_kg_per_m: 13.0,
        },
    ]
}

pub fn section_by_id(id: &str) -> Option<Section> {
    section_catalog()
        .into_iter()
        .find(|section| section.id == id)
}

pub fn section_family(id: &str) -> Option<&'static str> {
    let upper = id.trim().to_ascii_uppercase();
    for family in ["PFC", "RHS", "SHS", "CHS", "UB", "UC", "EA"] {
        if upper.ends_with(family) {
            return Some(family);
        }
    }
    None
}
