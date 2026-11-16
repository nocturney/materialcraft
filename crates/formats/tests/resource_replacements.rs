use materialcraft_core::Value;
use materialcraft_formats::sbs::{read_sbs, write_sbs};

fn node(filter: &str, parameter: &str, reference: &str) -> String {
    format!(
        r#"<graph><identifier v="Example"/><compNodes><compNode><uid v="7"/><compImplementation><compFilter><filter v="{filter}"/><parameters><parameter><name v="{parameter}"/><paramValue><constantValueString v="{reference}"/></paramValue></parameter></parameters></compFilter></compImplementation></compNode></compNodes></graph>"#
    )
}

fn resource(file: &str, format: &str, source: &str) -> String {
    format!(
        r#"<group><identifier v="Resources"/><content><resource><identifier v="asset"/><type v="bitmap"/><format v="{format}"/><filepath v="{file}"/>{source}</resource></content></group>"#
    )
}

fn package(content: &str, foreign: &str) -> String {
    format!(
        r#"<package><dependencies><dependency><filename v="?himself"/><uid v="1"/></dependency>{foreign}</dependencies><content>{content}</content></package>"#
    )
}

#[test]
fn surface_resources_use_library_replacements_before_embedded_or_external_data() {
    for (file, format, expected, filter, parameter, key) in [
        ("sbs://resources/sun.surface", "surface", "sun.exr", "bitmap", "bitmapresourcepath", "resource"),
        ("../textures/arbitrary.SURFACE", "surface", "arbitrary.exr", "bitmap", "bitmapresourcepath", "resource"),
    ] {
        // Valid old embedded data must not take precedence over the requested replacement
        let mut original = materialcraft_core::Package::new("Old");
        original.resources.push(materialcraft_core::Resource::new("asset", format!("asset.{format}"), materialcraft_core::ResourceBytes::new(vec![99])));
        let embedded = String::from_utf8(write_sbs(&original).unwrap()).unwrap();
        let embedded_doc = roxmltree::Document::parse(&embedded).unwrap();
        let source = embedded_doc.descendants().find(|n| n.has_tag_name("source")).unwrap();
        let xml = package(&(resource(file, format, &embedded[source.range()]) + &node(filter, parameter, "pkg:///Resources/asset?dependency=1")), "");
        let target = format!("sbs://resources/{expected}");
        let reads = std::cell::RefCell::new(Vec::new());
        let read = read_sbs(xml.as_bytes(), "elsewhere.sbs", &|path| {
            reads.borrow_mut().push(path.to_string());
            (path == target).then(|| vec![1, 2, 3])
        })
        .unwrap();

        assert!(read.warnings.is_empty(), "{:?}", read.warnings);
        assert_eq!(reads.borrow().as_slice(), std::slice::from_ref(&target));

        let imported = &read.pkg.resources[0];

        assert_eq!(imported.file, expected);
        assert_eq!(imported.source, target);
        assert_eq!(imported.bytes.as_slice(), &[1, 2, 3]);
        assert_eq!(read.pkg.graphs[0].nodes[0].params[key], Value::Text(imported.name.clone()));

        let saved = write_sbs(&read.pkg).unwrap();
        let reopened = read_sbs(&saved, "moved.sbs", &|_| None).unwrap();

        assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
        assert!(reopened.pkg.resources[0].file.ends_with(expected.rsplit_once('.').unwrap().1));
        assert_eq!(reopened.pkg.resources[0].bytes.as_slice(), &[1, 2, 3]);
        assert_eq!(reopened.pkg.graphs[0].nodes[0].params[key], Value::Text(imported.name.clone()));
    }
}

#[test]
fn missing_replacement_keeps_repairable_target_and_never_reads_the_old_file() {
    let xml = package(&resource("old/sun.surface", "surface", ""), "");
    let read = read_sbs(xml.as_bytes(), "example.sbs", &|path| {
        assert_eq!(path, "sbs://resources/sun.exr");

        None
    })
    .unwrap();

    assert_eq!(read.pkg.resources[0].file, "sun.exr");
    assert_eq!(read.pkg.resources[0].source, "sbs://resources/sun.exr");
    assert!(read.pkg.resources[0].bytes.is_empty());
    assert!(read.warnings.iter().any(|w| w.contains("sbs://resources/sun.exr")));

    let saved = String::from_utf8(write_sbs(&read.pkg).unwrap()).unwrap();

    assert!(!saved.contains(".surface"));
    assert!(saved.contains("sbs://resources/sun.exr"));
}

#[test]
fn external_copy_and_direct_bitmap_references_are_replaced_and_invalid_targets_are_not_read() {
    let external = r#"<source><externalcopy><filename v="old/shuf4_64_32.surface"/></externalcopy></source>"#;
    let xml = package(&(resource("", "surface", external) + &node("bitmap", "bitmapresourcepath", "old/shuf4_64_32.surface")), "");
    let read = read_sbs(xml.as_bytes(), "example.sbs", &|path| {
        assert_eq!(path, "sbs://resources/shuf4_64_32.exr");

        Some(vec![8])
    })
    .unwrap();

    assert!(read.warnings.is_empty(), "{:?}", read.warnings);
    assert!(read.pkg.resources.iter().all(|r| r.file == "shuf4_64_32.exr" && r.bytes.as_slice() == [8]));

    let xml = package(&resource("old/invalid%name.surface", "surface", ""), "");
    let read = read_sbs(xml.as_bytes(), "example.sbs", &|_| panic!("invalid target must not be read")).unwrap();

    assert!(read.pkg.resources[0].bytes.is_empty());
    assert!(read.pkg.resources[0].file.ends_with(".exr"));
    assert!(read.warnings.iter().any(|w| w.contains("invalid")));
}

#[test]
fn foreign_font_binding_is_imported_once_and_round_trips() {
    let dependency = r#"<dependency><filename v="nested/fonts.sbs"/><uid v="2"/></dependency>"#;
    let xml = package(&node("text", "fontdata", "pkg:///Resources/asset?dependency=2"), dependency);
    let foreign = package(&resource("../fonts/cour.ttf", "ttf", ""), "");
    let read = read_sbs(xml.as_bytes(), "example.sbs", &|path| match path {
        "nested/fonts.sbs" => Some(foreign.as_bytes().to_vec()),
        "sbs://resources/LiberationMono-Regular.ttf" => Some(vec![4, 5, 6]),
        _ => None,
    })
    .unwrap();

    assert!(read.warnings.is_empty(), "{:?}", read.warnings);
    assert_eq!(read.pkg.resources.len(), 1);
    assert_eq!(read.pkg.resources[0].file, "LiberationMono-Regular.ttf");
    assert_eq!(read.pkg.graphs[0].nodes[0].params["fontdata"], Value::Text("asset".into()));

    let reopened = read_sbs(&write_sbs(&read.pkg).unwrap(), "moved.sbs", &|_| None).unwrap();

    assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
    assert_eq!(reopened.pkg.graphs[0].nodes[0].params["fontdata"], Value::Text("asset".into()));
    assert_eq!(reopened.pkg.resources[0].bytes.as_slice(), &[4, 5, 6]);
}

#[test]
fn replacements_and_fallback_fonts_share_names_with_ordinary_resources() {
    for (file, format) in [("old/sun.surface", "surface"), ("missing.ttf", "ttf")] {
        let replacement = resource(file, format, "");
        let ordinary = resource("ordinary.png", "png", "");

        for content in [format!("{replacement}{ordinary}"), format!("{ordinary}{replacement}")] {
            let xml = package(&content, "");
            let read = read_sbs(xml.as_bytes(), "mixed.sbs", &|path| match path {
                "ordinary.png" => Some(vec![1]),
                "sbs://resources/sun.exr" | "sbs://resources/LiberationMono-Regular.ttf" => Some(vec![2]),
                _ => None,
            })
            .unwrap();

            assert!(read.warnings.is_empty(), "{:?}", read.warnings);
            assert_eq!(read.pkg.resources.len(), 2);
            assert_ne!(read.pkg.resources[0].name, read.pkg.resources[1].name);

            let reopened = read_sbs(&write_sbs(&read.pkg).unwrap(), "reopened.sbs", &|_| None).unwrap();

            assert!(reopened.warnings.is_empty(), "{:?}", reopened.warnings);
            for resource in &read.pkg.resources {
                assert_eq!(reopened.pkg.resource(&resource.name).unwrap().bytes, resource.bytes);
            }
        }
    }
}
