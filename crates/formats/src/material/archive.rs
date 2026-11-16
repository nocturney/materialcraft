use super::Manifest;
use materialcraft_core::Value;

use crate::{FormatError, MAX_JSON, MAX_RESOURCE, Result};

/// The portable material extension
pub const MATERIAL_EXT: &str = "mcmat";
/// `format` of every manifest
pub const FORMAT: &str = "craft-material";
/// Newest manifest version this build reads and writes
pub const VERSION: u32 = 2;
const SOURCE: &str = "source.sbs";

/// Whether maps of `usage` hold colors (stored sRGB-encoded) rather than data (linear)
pub fn srgb_usage(usage: &str) -> bool {
    matches!(
        usage,
        "baseColor"
            | "diffuse"
            | "emissive"
            | "specular"
            | "specularEdgeColor"
            | "scatteringColor"
            | "absorptionColor"
            | "coatColor"
            | "sheenColor"
            | "reflection"
            | "ambient"
    )
}

/// Whether other apps can set a value of this type from the manifest
pub fn portable(v: &Value) -> bool {
    matches!(
        v,
        Value::Float(_)
            | Value::Float2(_)
            | Value::Float3(_)
            | Value::Float4(_)
            | Value::Int(_)
            | Value::Int2(_)
            | Value::Bool(_)
            | Value::Color(_)
            | Value::Enum(_)
    )
}

/// Write a portable material: `manifest` (its `source` is set here), the encoded maps named by `manifest.maps[].file`, and the source package bytes
pub fn write_material(manifest: &Manifest, maps: &[(String, Vec<u8>)], source: &[u8]) -> Result<Vec<u8>> {
    let manifest = Manifest { source: SOURCE.into(), ..manifest.clone() };
    let json = serde_json::to_value(manifest).map_err(|e| FormatError::Io(e.to_string()))?;
    let shared: craft_mcmat::Manifest = serde_json::from_value(json).map_err(|e| FormatError::Io(e.to_string()))?;
    craft_mcmat::write(&shared, maps, source).map_err(|e| FormatError::Io(e.to_string()))
}

/// Read a portable material's manifest and source package bytes
pub fn read_material(bytes: &[u8]) -> Result<(Manifest, Vec<u8>)> {
    let bad = |e: craft_mcmat::Error| FormatError::BadPackage(format!("portable material: {e}"));
    let limits = craft_mcmat::Limits {
        min_version: VERSION,
        max_version: VERSION,
        max_entries: 100_000,
        max_maps: 100_000,
        max_manifest_bytes: MAX_JSON,
        max_entry_bytes: MAX_RESOURCE,
        max_total_bytes: MAX_JSON + MAX_RESOURCE,
    };
    let mut material = craft_mcmat::Material::open(bytes, limits).map_err(bad)?;
    // Consume nested values individually to avoid copying the whole untrusted JSON manifest.
    let parameters = std::mem::take(&mut material.manifest.parameters)
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| FormatError::BadPackage(format!("portable material: {e}")))?;
    let inputs = std::mem::take(&mut material.manifest.inputs)
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| FormatError::BadPackage(format!("portable material: {e}")))?;
    let presets = std::mem::take(&mut material.manifest.presets)
        .into_iter()
        .map(serde_json::from_value)
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| FormatError::BadPackage(format!("portable material: {e}")))?;
    let maps = std::mem::take(&mut material.manifest.maps)
        .into_iter()
        .map(|map| {
            let mut fields = map.extra;
            fields.insert("usage".into(), map.usage.into());
            fields.insert("file".into(), map.file.into());
            serde_json::from_value(serde_json::Value::Object(fields.into_iter().collect()))
        })
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| FormatError::BadPackage(format!("portable material: {e}")))?;
    let shared = &material.manifest;
    let manifest = Manifest {
        format: shared.format.clone(),
        version: shared.version,
        name: shared.name.clone(),
        graph: shared.graph.clone(),
        size: shared.size,
        maps,
        parameters,
        inputs,
        presets,
        source: shared.source.clone(),
    };
    let source = material.read_source().map_err(bad)?;
    Ok((manifest, source))
}

/// Whether `bytes` look like a portable material (a zip with `material.json`)
pub fn is_material(bytes: &[u8]) -> bool {
    craft_mcmat::is_material(bytes, 100_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{ImageInput, MapEntry, Parameter};
    use materialcraft_core::ExposedParameter;

    fn manifest() -> Manifest {
        let param = ExposedParameter {
            description: String::new(),
            editor: None,
            clamp: false,
            step: None,
            id: "rough".into(),
            label: "Roughness".into(),
            group: String::new(),
            default: Value::Float(0.5),
            min: 0.0,
            max: 1.0,
            options: Vec::new(),
        };
        Manifest {
            format: FORMAT.into(),
            version: VERSION,
            name: "Bricks".into(),
            graph: "Bricks".into(),
            size: [64, 64],
            maps: vec![MapEntry { usage: "baseColor".into(), file: "maps/baseColor.png".into(), color_space: "srgb".into() }],
            parameters: vec![Parameter { param, value: Value::Float(0.25) }],
            inputs: vec![ImageInput { identifier: "curvature".into(), kind: "gray".into() }],
            presets: Vec::new(),
            source: String::new(),
        }
    }

    #[test]
    fn round_trips_and_rejects_newer_versions() {
        let m = manifest();
        let bytes = write_material(&m, &[("maps/baseColor.png".into(), vec![1, 2, 3])], b"pkg").unwrap();
        assert!(is_material(&bytes));
        let (back, source) = read_material(&bytes).unwrap();
        assert_eq!(back, Manifest { source: SOURCE.into(), ..m.clone() }, "the writer names the source entry");
        assert_eq!(source, b"pkg");
        // Parameters are flat objects with the value in the API's typed form
        let json = serde_json::to_value(&m).unwrap();
        assert_eq!(json["parameters"][0]["id"], "rough");
        assert_eq!(json["parameters"][0]["value"], serde_json::json!({"t": "float", "v": 0.25}));

        // Version 1 embeds the old JSON-based .mcraft source, which this app's current
        // SBS package reader cannot open; do not claim that accepting its manifest is enough.
        let older = Manifest { version: 1, ..m.clone() };
        assert!(read_material(&write_material(&older, &[], b"").unwrap()).is_err());
        let newer = Manifest { version: VERSION + 1, ..m };
        assert!(read_material(&write_material(&newer, &[], b"").unwrap()).is_err());
        assert!(!is_material(b"not a zip"));
        assert!(read_material(b"garbage").is_err());
    }
}
